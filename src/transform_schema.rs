use openapiv3::{
    IndexMap, OpenAPI, Operation, Parameter, RefOr, ReferenceOr, RequestBody,
    Responses, Schema,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

type SectionName = String;
type Method = String;
type PathName = String;
type SectionEntry = BTreeMap<PathName, PathMethods>;
type PathMethods = BTreeMap<Method, PathInfo>;

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct TransformedSchema {
    pub info: Info,
    pub sections: BTreeMap<SectionName, SectionEntry>,
    pub schemas: IndexMap<String, ReferenceOr<Schema>>,
}

/*
info:
    title:
    description:
    termsOfService:
    version:
sections:
    section_name:
        - /path:
            get:
                summary:
                description:
                operation_id:
                parameters: []
*/

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Info {
    pub title: String,
    pub description: Option<String>,
    pub terms_of_service: Option<String>,
    pub version: String,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct PathInfo {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation_id: Option<String>,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RefOr<RequestBody>>,
    pub responses: Responses,
}

#[allow(dead_code)]
pub enum HttpMethod {
    Get,
    Post,
    Delete,
    Patch,
    Put,
}

fn extract_path_info(item: &Operation) -> (String, PathInfo) {
    let tag = item.tags.first().cloned().unwrap_or_else(|| "Other".into());

    let parameters = item
        .parameters
        .iter()
        .map(|param| {
            let RefOr::Item(param) = param else { panic!() };
            param.clone()
        })
        .collect();
    (
        tag,
        PathInfo {
            summary: item.summary.clone(),
            description: item.description.clone(),
            operation_id: item.operation_id.clone(),
            parameters,
            request_body: item.request_body.clone(),
            responses: item.responses.clone(),
        },
    )
}

pub fn transform_schema(schema: &OpenAPI) -> TransformedSchema {
    let mut transformed = TransformedSchema::default();

    transformed.info.title.clone_from(&schema.info.title);
    transformed
        .info
        .description
        .clone_from(&schema.info.description);
    transformed
        .info
        .terms_of_service
        .clone_from(&schema.info.terms_of_service);
    transformed.info.version.clone_from(&schema.info.version);

    for (path_name, path_item) in &schema.paths.paths {
        let RefOr::Item(path_item) = path_item else {
            panic!("References not supported in path items");
        };

        if let Some(get) = &path_item.get {
            let (tag, info) = extract_path_info(get);
            let section = transformed.sections.entry(tag).or_default();
            let path_bit = section.entry(path_name.to_string()).or_default();
            path_bit.insert("get".to_string(), info);
        }

        if let Some(post) = &path_item.post {
            let (tag, info) = extract_path_info(post);
            let section = transformed.sections.entry(tag).or_default();
            let path_bit = section.entry(path_name.to_string()).or_default();
            path_bit.insert("post".to_string(), info);
        }

        if let Some(put) = &path_item.put {
            let (tag, info) = extract_path_info(put);
            let section = transformed.sections.entry(tag).or_default();
            let path_bit = section.entry(path_name.to_string()).or_default();
            path_bit.insert("put".to_string(), info);
        }

        if let Some(patch) = &path_item.patch {
            let (tag, info) = extract_path_info(patch);
            let section = transformed.sections.entry(tag).or_default();
            let path_bit = section.entry(path_name.to_string()).or_default();
            path_bit.insert("patch".to_string(), info);
        }

        if let Some(delete) = &path_item.delete {
            let (tag, info) = extract_path_info(delete);
            let section = transformed.sections.entry(tag).or_default();
            let path_bit = section.entry(path_name.to_string()).or_default();
            path_bit.insert("delete".to_string(), info);
        }
    }

    transformed.schemas = schema.components.schemas.clone().into();

    transformed
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    const SAMPLE:&str = "openapi: 3.0.0
info:
  title: Docker Remote API
  description: The API for each docker installation.
  termsOfService: 'http://example.com/tos/'
  version: v1.21
paths:
  /containers/json:
    get:
      summary: List containers
      description: List containers
      operationId: findAllContainers
      parameters:
        - name: all
          in: query
          description: >-
            Show all containers. Only running containers are shown by default
            (i.e., this defaults to false)
          schema:
            type: boolean
            default: false
        - name: limit
          in: query
          description: 'Show  last created containers, include non-running ones.'
          schema:
            type: integer
        - name: since
          in: query
          description: 'Show only containers created since Id, include non-running ones.'
          schema:
            type: string
        - name: before
          in: query
          description: 'Show only containers created before Id, include non-running ones.'
          schema:
            type: string
        - name: size
          in: query
          description: '1/True/true or 0/False/false, Show the containers sizes.'
          schema:
            type: boolean
        - name: filters
          in: query
          description: >-
            A JSON encoded value of the filters (a map[string][]string) to
            process on the containers list
          schema:
            type: array
            items:
              type: string
      responses:
        '200':
          description: no error
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/ContainerConfig'
            text/plain:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/ContainerConfig'
        '400':
          description: bad parameter
        '500':
          description: server error
      tags:
        - Container
";

    #[test]
    fn test_transform() {
        let schema = serde_yaml::from_str(SAMPLE).unwrap();
        let transformed = transform_schema(&schema);
        let expected = serde_yaml::from_str::<TransformedSchema>(
            "
info:
  title: Docker Remote API
  description: The API for each docker installation.
  terms_of_service: 'http://example.com/tos/'
  version: v1.21
sections:
  Container:
    /containers/json:
        get:
          summary: List containers
          description: List containers
          operation_id: findAllContainers
          parameters:
          - name: all
            description:  >-
                Show all containers. Only running containers are shown by default
                (i.e., this defaults to false)
            in: query
            schema:
                type: boolean
                default: false
          - name: limit
            in: query
            description: 'Show  last created containers, include non-running ones.'
            schema:
              type: integer
          - name: since
            in: query
            description: 'Show only containers created since Id, include non-running ones.'
            schema:
              type: string
          - name: before
            in: query
            description: 'Show only containers created before Id, include non-running ones.'
            schema:
              type: string
          - name: size
            in: query
            description: '1/True/true or 0/False/false, Show the containers sizes.'
            schema:
              type: boolean
          - name: filters
            in: query
            description: >-
              A JSON encoded value of the filters (a map[string][]string) to
              process on the containers list
            schema:
              type: array
              items:
                type: string
            ",
        )
        .unwrap();
        assert_eq!(transformed, expected);
    }
}
