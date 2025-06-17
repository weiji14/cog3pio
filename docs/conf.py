# General configurations.
needs_sphinx = "6.2"
extensions = [
    "myst_parser",
    "sphinx_ext_mystmd",
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
]
# Options for source files.
exclude_patterns = [
    "_build",
]
# Options for napoleon.
# https://www.sphinx-doc.org/en/master/usage/extensions/napoleon.html#module-sphinx.ext.napoleon
napoleon_use_admonition_for_examples = True
napoleon_use_rtype = False
napoleon_use_ivar = True
