#!/bin/sh
# Set color based on issue event action
case "${GITHUB_EVENT_ACTION}" in
    opened)  echo "issue_color=#57F287" >> "$GITHUB_OUTPUT" ;;   # green
    closed)  echo "issue_color=#ED4245" >> "$GITHUB_OUTPUT" ;;   # red
    reopened) echo "issue_color=#FFFF00" >> "$GITHUB_OUTPUT" ;; # yellow
    *)       echo "issue_color=#959B96" >> "$GITHUB_OUTPUT" ;;   # gray
esac
