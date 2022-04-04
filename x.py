#!/bin/sh
# This is simultaneously a valid shell script and python file.
# If executed as a shell script, it attempts to re-execute
# itself as python. (Note that the MSYSTEM check used to be
# required for mingw but may not be required anymore)
''':' && if [ ! -z "$MSYSTEM" ] ; then exec python "$0" "$@" ; else which python3 > /dev/null 2> /dev/null && exec python3 "$0" "$@" || exec python "$0" "$@" ; fi
'''

# This file is only a "symlink" to bootstrap.py, all logic should go there.

import os
import sys

# If this is python2, check if python3 is available and re-execute with that
# interpreter.
if sys.version_info.major < 3:
    try:
        # On Windows, `py -3` sometimes works.
        # Try this first, because 'python3' sometimes tries to launch the app
        # store on Windows
        os.execvp("py", ["py", "-3"] + sys.argv)
    except OSError:
        try:
            os.execvp("python3", ["python3"] + sys.argv)
        except OSError:
            # Python 3 isn't available, fall back to python 2
            pass

rust_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.append(os.path.join(rust_dir, "src", "bootstrap"))

import bootstrap
bootstrap.main()
