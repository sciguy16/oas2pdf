#let title = [ {{ info.title }} ]

#set heading(numbering: "1.")
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
=== {{ method_name }} - {{ method.operation_id }}
{{ method.summary }}

{{ method.description }}

==== Parameters
#table(
	columns: 4,
	[Name], [Required], [Description], [Schema],
{% for param in method.parameters %}
	[{{ param.name }}],
	[{{ param.required|default(value="false") }}],
	[{{ param.description|default(value="") }}],
	[{{ param.format|default(value="") }}],
{% endfor %}{# param #}
)

{% if method.request_body %}
==== Request body
{% if method.request_body is reference %}
REF: {{ method.request_body["$ref"] }}
{% else %}
{{ method.request_body.description|default(value="") }}


#table(
	columns: 2,
{% for field,ty in method.request_body.content %}
	[{{ field }}],
	[{{ ty.schema|json_encode() }}],
{% endfor %}{# field,ty #}
)

{% endif %}

{%- endif -%}{# body #}

{% endfor %}{# (method, details) #}
{% endfor %}{# (path_name, path) #}
{% endfor %}{# (path, item) #}
