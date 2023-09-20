## Design

### Launcher

What should launcher do:
1. Read the config and modify the config based on the current environment
2. Launch the application executable.
3. Inject the payload dll into the executable
4. Pass the config to dll


### Data

What should the data.dll to do:
1. Get data from launcher
   1. Through named pipe 
   2. Through exported function
2. Perform injections