#!/usr/bin/bash

cwd="/etc/olympus/chronos/logs"

cd "${cwd}"

if [[ $? -ne 0 ]]
then
	echo "Failed to set '${cwd}' as work directory"
	exit 1
fi

# Zip every log file in the logs directory
date +'%Y%m%d%H%M%S' | awk '{print "zip log_dump_" $1 ".zip *.log"}' | bash

if [[ $? -ne 0 ]]
then
	echo "Failed to zip *.lpg files in ${cwd}"
	exit 1
fi

# Delete all log files
ls -ltra | grep -E "\.log$" | awk '{print "rm -f " $9}' | bash

if [[ $? -ne 0 ]]
then
	echo "Failed to delete *.log files in ${cwd}"
	exit 1
fi

# Keep only the last 5 archived zip file
ls -ltra | grep -E "\.zip$" | awk '{print $9}' | head -n -5 | awk '{print "rm -f " $1}' | bash

if [[ $? -ne 0 ]]
then
	echo "Failed to cleanup log files in ${cwd}"
	exit 1
fi

ls -l

exit 0

