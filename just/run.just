GAMES_FOLDER:="/home/ufo/Temp/Games/bg"

## RESOURCE BROWSER

# Run Infinitier Explorer with the games listed in config_test/infinitier_explorer.toml
[group('run_explorer')]
run_explorer:
  cargo run -p infinitier_explorer -- --config config_test/infinitier_explorer.toml


## SAVE GAME EDITOR 

# Run Infinitier Keeper with the games listed in config_test/infinitier_keeper.toml
[group('run_keeper')]
run_keeper:
  cargo run -p infinitier_keeper -- --config config_test/infinitier_keeper.toml