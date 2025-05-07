#let title = [ {{ info.title }} ]

#let numbering-limited(max-level, schema) = (..numbers) => {
  if numbers.pos().len() <= max-level {
    numbering(schema, ..numbers)
  }
}
#set heading(numbering: numbering-limited(3, "1."))
#let appendix(body) = {
  set heading(numbering: numbering-limited(1, "A."), supplement: [Appendix])
  counter(heading).update(0)
  body
}

#set page(
  header: align(
    right + horizon,
    title
  ),
  numbering: "1/1"
)

#align(horizon)[
	#align(center, text(27pt)[
	  *#title*
	])
	#align(center, text(24pt)[ {{info.description}} ])
]

#pagebreak()

#outline(
	depth: 2,
	indent: auto,
)

{% for header,section in sections %}
#pagebreak()
= {{ header }}
{% for path_name,path in section %}
== {{ path_name}}
{% for method_name,method in path %}
=== {{ method_name|upper }} - {{ method.operation_id }}
{{ method.summary }}

{{ method.description }}

{% if method.request_body and method.request_body.content %}
{% for mediatype,meta in method.request_body.content %}
- {{ mediatype }}: {{ meta.schema|ref|safe }}
{% endfor %}
{% endif %}

{% if method.parameters %}
==== Parameters
{% for param in method.parameters %}
===== {{param.in}}: {{param.name}} {% if param.required %}{{"(required)"}}{% endif %}

{{ param.description|default(value="") }}

- type: {{ param.schema|ref|safe}} {{param.schema.type|default(value="")}}
{% if param.style %}- style: {{param.style}}{% endif %}
{% if param.schema.format %}- format: {{param.schema.format}}{% endif %}
{% if param.schema.nullable %}- nullable{% endif %}

{% endfor %}{# param #}
{% endif %}{# param #}

==== Response
{% for code,response in method.responses %}
==== {{ code }}
{{ response.description|default(value="") }}

{% if response.content %}
{% for mediatype,meta in response.content %}
- {{ mediatype }}: {{ meta.schema|ref|safe }}{{ meta.schema|show_type|safe }}
{% endfor %}{# (mediatype,meta) #}
{% endif %}{# response.content %}

{% endfor %}{# (code,response) #}

{% endfor %}{# (method_name, method) #}
{% endfor %}{# (path_name, path) #}
{% endfor %}{# (path, item) #}

{% endfor %}{# (header, section) #}

#pagebreak()
#show: appendix

= Schemas

{% for name,schema in schemas %}
== {{name}}<{{name}}>

{{ schema.description|default(value="") }}


{% if schema.enum %}
Allowed values:
{% for value in schema.enum %}
- `{{value|safe}}`
{% endfor %}
{% endif %}

{% if schema.properties %}
{% for prop_name,prop in schema.properties %}
=== `{{prop_name|safe}}`#h(1fr){{prop|show_type|safe}}{{prop|ref|safe}}
{{ prop.description|default(value="") }}
{% if prop.format %}- format: {{prop.format}}{% endif %}
{% if prop.nullable %}- nullable{% endif %}
{% if prop.additionalProperties %}- additional properties: {{prop.additionalProperties|ref|safe}}{% endif %}

{% endfor %}{# (prop_name, prop) #}
{% endif %}{# schema.properties #}

{% if schema.oneOf %}
One of:
{% for variant in schema.oneOf %}
- `{{variant.type}}`: {{variant.description}}
{% if variant.properties %}
{% for name,prop in variant.properties %}
  - `{{name}}`: `{{prop.type|default(value="")}}`{{prop|ref|safe}}
{% endfor %}{# (name, prop) #}
{% endif %}{# variant.properties #}
{% endfor %}{# variant #}
{% endif %}{# schema.oneOf #}

{% endfor %}{# (name, schema) #}
// required schema. Don't list them, but do a lookup and mark the field as required
 

