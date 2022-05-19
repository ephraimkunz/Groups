# Groups
Student and instructor facing website that allow students to provide availability and instructors to divide students into groups based on availability.

## Live Site
The site is live at https://groups-server.shuttleapp.rs/student for students and https://groups-server.shuttleapp.rs/instructor for instructors.

## Support
Questions or issues can be resolved by clicking on the issues tab above an opening a new issue, or by sending me an email at kunzep@byui.edu.

## Development Information

### Run local webserver for testing
Run `build_and_test_site_local.sh` in project root. A local webserver will be spun up and the IP address + port will be printed to the console.

### Deploy
Run `build_and_deploy_site.sh` in project root.

### Endpoints
BASE_URL is printed as a result of deploying or running the local webserver.
* BASE_URL/student. Students fill out the form there and send the instructor the schedule code they receive, either through email, a survey, etc.
* BASE_URL/instructor. Instructors paste in all of the student schedule codes they receive. They can use the interface to manually split into groups based on availability, or have it done automatically.
* BASE_URL/random (local webserver only). Get 50 random schedule codes for testing instructor functionality.
  
