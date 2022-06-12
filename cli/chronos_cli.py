#!/usr/bin/python3

import sys
import os
import getpass
import subprocess
import socket

valid_verbs = ["list", "enable", "disable", "show", "log"]
root_dir = "/etc/olympos/chronos"

# If not root running the script, then exit
if getpass.getuser() != "root":
    print("You must be root to running this command")
    exit(2)

# Assume that proper number of input are provided
if len(sys.argv) < 3:
    print("Not enough input parameter")
    print("Formats:")
    print(sys.argv[0], "list timer-set")
    print(sys.argv[0], "list all timers")
    print(sys.argv[0], "list active timers")
    print(sys.argv[0], "show <timer-name> file")
    print(sys.argv[0], "show <timer-name> status")
    print(sys.argv[0], "log <timer-name>")
    print(sys.argv[0], "enable timer <timer-name>")
    print(sys.argv[0], "disable timer <timer-name>")
    exit(1)

if sys.argv[1] not in valid_verbs:
    print("Invalid verb:", sys.argv[1])
    exit(1)

#
# Handle list requests
#
if sys.argv[1] == "list":
    #
    # List timer sets
    #
    if sys.argv[2] == "timer-sets":
        files = os.listdir(root_dir + "/all_timers/")
        sets = []

        for file in files:
            qual = file.split(".")
            if qual[0] not in sets:
                sets.append(qual[0])
        
        sets.sort()
        for set in sets:
            print(set)
        exit(0)

    #
    # Reactions from here will have 3 parameters
    #
    if len(sys.argv) < 4:
        print("Missing parameter")
        exit(1)

    #
    # List timer files
    #
    if (sys.argv[2] == "all" or sys.argv[2] == "active") and sys.argv[3] == "timers":
        files = os.listdir(root_dir + "/" + sys.argv[2] + "_timers/")
        timers = []

        for file in files:
            qual = file.split(".")
            if qual[len(qual) - 1] == "conf":
                name = ".".join(qual[0:len(qual) - 1:])
                timers.append(name)

        timers.sort()
        for timer in timers:
            print(timer)
        exit(0)

    print("Invalid list parameter")
    exit(0)


#
# Show commands
#
if sys.argv[1] == "show":
    #
    # Reactions from here will have 3 parameters
    #
    if len(sys.argv) < 4:
        print("Missing parameter")
        exit(1)

    #
    # List content of file
    #
    if sys.argv[3] == "file":
        os.chdir(root_dir + "/all_timers")
        file = open(sys.argv[2] + ".conf", mode="r")
        content = file.read()
        file.close()
        print(content)

    #
    # List status from Hermes
    #
    if sys.argv[3] == "status":
        hermes_status = os.popen("systemctl status olympos.hermes | grep \"active (running)\"").read()
        if hermes_status == "":
            print("Hermes is nto running, no record available")
            exit(1)

        groups = os.popen("/usr/local/bin/hermes-cli list-groups").read()
        groups_list = groups.split("\n")

        if "timer" in groups_list:
            content = os.popen("/usr/local/bin/hermes-cli get-item timer/" + sys.argv[2]).read()
            content = content.split("\n")
            if content[0] == "200":
                print(content[3])
            else:
                print("Timer is not enabled")

    exit(0)

#
# Handle logs
#
if sys.argv[1] == "log":
    file = root_dir + "/logs/" + sys.argv[2] + ".log"
    if os.path.exists(file):
        os.system("/usr/bin/less " + file)
    else:
        print("Log cannot be found:", file)

#
# Handle enable request
#
if sys.argv[1] == "enable":
    # If link does exist, it means that timer is already enabled, nothing to do
    # If link does not exist, then it must be created
    os.chdir(root_dir + "/active_timers")

    if os.path.exists(root_dir + "/active_timers/" + sys.argv[2] + ".conf"):
        print("OK Already enabled")
        exit(0)

    status = os.symlink("../all_timers/" + sys.argv[2] + ".conf", sys.argv[2] + ".conf")
    print("OK Done")
    exit(0)

#
# Handle disable request
#
if sys.argv[1] == "disable":
    os.chdir(root_dir + "/active_timers")

    if not os.path.exists(root_dir + "/active_timers/" + sys.argv[2] + ".conf"):
        print("OK Already disabled")
        exit(0)

    os.remove(sys.argv[2] + ".conf")

    print("OK Done")
    exit(0)