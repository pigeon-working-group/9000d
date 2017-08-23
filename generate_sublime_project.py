from json import dump

env = {
	"TARGET_ADDRESS": input("Target address: ") or "pigeon.local",
	"TARGET_BIN_LOCATION": input("Target binary location: ") or "~",
	"TARGET_USER": input("Target user: ") or "philip",
	"VM_PORT": input("Virtual machine port: ") or "2222",
	"VM_PROJECT_LOCATION": input("Virtual machine project location: ") \
							or "/media/sf_pigeond",
	"VM_USER": input("Virtual machine user: ") or "philip",
	"COLOR": "never"
}

if not env["TARGET_BIN_LOCATION"].endswith("/"):
	env["TARGET_BIN_LOCATION"] = "%s/" % env["TARGET_BIN_LOCATION"]

dump(
	{
		"build_systems":
		[
			{
				"shell_cmd": "make",
				"env": env,
				"name": "Pigeon",
				"working_dir": "${project_path}",
				"variants": 
				[
					{
						"name": "Run",
						"shell_cmd": "ssh %s@%s '%spigeond'" % (
							env["TARGET_USER"], env["TARGET_ADDRESS"],
							env["TARGET_BIN_LOCATION"])
					},
					{
						"name": "Stop",
						"shell_cmd": "ssh %s@%s 'killall pigeond'" % (
							env["TARGET_USER"], env["TARGET_ADDRESS"])
					}
				]
			}
		],
		"folders":
		[
			{
				"path": "."
			}
		],
		"settings":
		{
			"tab_size": 4
		}
	}, 
	open("pigeond.sublime-project", "w"), indent=4, sort_keys=True
)
