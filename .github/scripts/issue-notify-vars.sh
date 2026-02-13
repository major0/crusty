#!/bin/sh
# Set color based on issue event action
case "${GITHUB_EVENT_ACTION}" in
    opened)  echo "issue_color=5763719" >> "$GITHUB_OUTPUT" ;;   # green
    closed)  echo "issue_color=15548997" >> "$GITHUB_OUTPUT" ;;  # red
    reopened) echo "issue_color=16776960" >> "$GITHUB_OUTPUT" ;; # yellow
    *)       echo "issue_color=9807270" >> "$GITHUB_OUTPUT" ;;   # gray
esac
