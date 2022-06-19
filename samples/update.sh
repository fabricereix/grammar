#!/bin/bash


cat <<END >grammar.html
<link rel="stylesheet" type="text/css" href="style.css">
END
grammar < grammar.grammar >>grammar.html


cat <<END >address.html
<link rel="stylesheet" type="text/css" href="style.css">
END
grammar < address.grammar>>address.html

