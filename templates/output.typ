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

{% for (header, section) in schema.sections.iter() %}

= {{ header }}

{% for (path_name, path) in section.iter() %}
== {{ path_name}}

{% for (method_name, method) in path.iter() %}
=== {{ method_name }}
{{ method.summary.as_ref().unwrap() }}
{% endfor %}

{% endfor %}{# (method, details) #}

{% endfor %}{# (path, item) #}
