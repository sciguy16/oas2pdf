use openapiv3::{OpenAPI, Parameter, RefOr};
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
}

#[allow(dead_code)]
pub enum HttpMethod {
    Get,
    Post,
    Delete,
    Patch,
    Put,
}

pub fn transform_schema(schema: &OpenAPI) -> TransformedSchema {
    let mut transformed = TransformedSchema::default();

    transformed.info.title = schema.info.title.clone();
    transformed.info.description = schema.info.description.clone();
    transformed.info.terms_of_service = schema.info.terms_of_service.clone();
    transformed.info.version = schema.info.version.clone();

    for (path_name, path_item) in &schema.paths.paths {
        let RefOr::Item(path_item) = path_item else {
            panic!("References not supported in path items");
        };

        if let Some(get) = &path_item.get {
            let tag =
                get.tags.first().cloned().unwrap_or_else(|| "Other".into());
            let section = transformed.sections.entry(tag).or_default();
            let path_bit = section.entry(path_name.to_string()).or_default();
            let parameters = get
                .parameters
                .iter()
                .map(|param| {
                    let RefOr::Item(param) = param else { panic!() };
                    param.clone()
                })
                .collect();
            path_bit.insert(
                "get".to_string(),
                PathInfo {
                    summary: get.summary.clone(),
                    description: get.description.clone(),
                    operation_id: get.operation_id.clone(),
                    parameters,
                },
            );
        }
    }

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
