#!/bin/sh
# Set color and state based on PR event action
if [ "${GITHUB_EVENT_ACTION}" = "closed" ] && [ "${PR_MERGED}" = "true" ]; then
    echo "pr_color=10181046" >> "$GITHUB_OUTPUT"  # purple
    echo "pr_state=merged" >> "$GITHUB_OUTPUT"
elif [ "${GITHUB_EVENT_ACTION}" = "closed" ]; then
    echo "pr_color=15548997" >> "$GITHUB_OUTPUT"  # red
    echo "pr_state=closed" >> "$GITHUB_OUTPUT"
elif [ "${GITHUB_EVENT_ACTION}" = "opened" ]; then
    echo "pr_color=5763719" >> "$GITHUB_OUTPUT"   # green
    echo "pr_state=opened" >> "$GITHUB_OUTPUT"
else
    echo "pr_color=9807270" >> "$GITHUB_OUTPUT"   # gray
    echo "pr_state=${GITHUB_EVENT_ACTION}" >> "$GITHUB_OUTPUT"
fi
