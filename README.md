# Groups
Student and instructor facing website that allow students to provide availability and instructors to divide students into groups based on availability.

## Live Site
The site is live at https://groups-server.shuttleapp.rs/student for students and https://groups-server.shuttleapp.rs/instructor for instructors.

## Support
Questions or issues can be resolved by clicking on the issues tab above an opening a new issue, or by sending me an email at kunzep@byui.edu.

## Development Information

### Deploy
Run `build_and_deploy_site.sh` in project root.

### Usage
BASE_URL is printed as a result of deploying.
* Students: Visit BASE_URL/student. Fill out the form there and send your instructor the schedule code you receive, either through email, a survey, etc.
* Instructors: Visit BASE_URL/instructor. Paste in all of the student schedule codes you received. You can use the interface to manually split into groups based on availability, or have it done automatically.
  
