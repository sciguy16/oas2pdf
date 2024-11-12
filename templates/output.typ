#let title = [ {{ schema.info.title }} ]


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
	{% if let Some(description) = schema.info.description %}
	#align(center, text(24pt)[ {{description}} ])
	{% endif %}
]

#pagebreak()

#outline(
	depth: 2,
	indent: auto,
)

{% for (header, section) in schema.sections.iter() %}

#pagebreak()
= {{ header }}

{% for (path_name, path) in section.iter() %}
== {{ path_name}}

{% for (method_name, method) in path.iter() %}
=== {{ method_name }} - {{ method.operation_id.as_ref().unwrap() }}
{{ method.summary.as_ref().unwrap() }}

{{ method.description.as_ref().unwrap() }}

==== Parameters
#table(
	columns: 4,
	[Name], [Required], [Description], [Schema],

{% for param in method.parameters.iter() %}

	[{{ param.name }}],
	[{{ param.required }}],
	[{{ param.description.as_ref().unwrap() }}],
	[{{ param.format|fmt("{:?}") }}], 
{% endfor %}{# param #}
)

{% endfor %}{# (method, details) #}

{% endfor %}{# (path_name, path) #}

{% endfor %}{# (path, item) #}
