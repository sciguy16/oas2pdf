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
{% endfor %}{# (method, details) #}
{% endfor %}{# (path_name, path) #}
{% endfor %}{# (path, item) #}
