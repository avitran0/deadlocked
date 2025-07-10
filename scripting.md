# lua scripting

## functions

### register_once(func: function): void

registers a function to run once on script load

### register_tick(func: function): void

registers a function to run every tick (100 times/second)

### register_key_held(key: int, func: function): void

registers a function to run as long as a certain key is held

### register_key_pressed(key: int, func: function): void

registers a function to run once when a key has been pressed
