# Groups
Student and instructor facing website that allow students to provide availability and instructors to divide students into groups based on availability.

* Students get to advertise their availability in their own timezone. No need for timezone conversions.
* Instructors can manually group students based on a grid layout, or use an automated scheduler with a configurable group size that seeks to maximize the number of hours all members of the group are available.

## Student Site
Empty           |  Filled Out
:-------------------------:|:-------------------------:
![](https://user-images.githubusercontent.com/10914093/169444643-27cbbf38-682b-45a1-b22c-9d5b0d93f5f9.png)  |  ![](https://user-images.githubusercontent.com/10914093/169444640-6ec85094-93fc-42e8-9acc-985d88261cf6.png)

## Instructor Site
Empty           |  Filled Out
:-------------------------:|:-------------------------:
![](https://user-images.githubusercontent.com/10914093/169445317-185c2eb1-2d6c-472e-92c4-6f87e5070017.png)  |  ![](https://user-images.githubusercontent.com/10914093/169445327-ad1476fa-6719-469c-8653-80de64549bfe.png)

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

### How it works
The core group scheduling code is written in Rust and runs in the browser after being compiled to WebAssembly. This code also handles encoding and decoding schedule ids (base64 encoded strings that compactly encode student information and a bitvector of student scheduling information). The group scheduler uses a hill-climbing algorithm with random re-starts to avoid getting stuck in a local minima. It creates a random group assignment, then randomly swaps students as long as a swap results in a better objective function for the entire group assignment. 

I considered other search algorithms (simulated annealing, genetic search, etc) and constraint solvers (this problem's formulation is similar to the wedding seating problem) but the main barrier lies in implementing a better objective function. This function should maximize the number of hours (especially consecutive hours) each team members in a group have in common, while attempting to make all groups equally good (we don't want some very good groups that maximize the objective function but that overshadow some very bad groups). It's possible the Gini coefficient is how we could approach this. With a better objective function, we could use a more sophisticated search algorithm to attempt to maximize it. As it is, the current hill-climbing methodology finds the best possible group assignment relatively quickly, as shown by plotting the convergence in unit tests with random data. Real student data is not random so it remains to be seen how this will perform in the real world.
  
