#!/bin/bash
set -uo pipefail

CHANGELOG_FILE="${1:-CHANGELOG.md}"
REACT_SDK_CHANGELOG="packages/react-sdk/CHANGELOG.md"

if [ "${NO_CHANGELOG_LABEL}" = "true" ]; then
    # 'no changelog' set, so finish successfully
    echo "\"no changelog\" label has been set"
    exit 0
else
    # a changelog check is required
    # pass if either the root or the react-sdk changelog has been updated
    if ! git diff --exit-code "origin/${BASE_REF}" -- "${CHANGELOG_FILE}" > /dev/null 2>&1; then
        echo "The \"${CHANGELOG_FILE}\" file has been updated."
        exit 0
    fi

    if ! git diff --exit-code "origin/${BASE_REF}" -- "${REACT_SDK_CHANGELOG}" > /dev/null 2>&1; then
        echo "The \"${REACT_SDK_CHANGELOG}\" file has been updated."
        exit 0
    fi

    >&2 echo "Changes should come with an entry in the \"CHANGELOG.md\" or \"${REACT_SDK_CHANGELOG}\" file. This behavior
can be overridden by using the \"no changelog\" label, which is used for changes
that are trivial / explicitly stated not to require a changelog entry."
    exit 1
fi
