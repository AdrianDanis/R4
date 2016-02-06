#!/bin/sh
rustdoc --no-defaults --passes collapse-docs --passes unindent-comments "$@"
