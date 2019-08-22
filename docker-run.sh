#!/bin/sh
diesel setup
diesel migration run
upowdb-backend -vvv
