#!/bin/sh
export DATABASE_URL=app.db
diesel setup
diesel migration run
