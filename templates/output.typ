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

{% for (path, item) in schema.paths.iter() %}

= {{ path }}
game

{% endfor %}
