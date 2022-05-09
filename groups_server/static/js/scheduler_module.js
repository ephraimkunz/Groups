import init, { timezones_wasm, Student, tz_groups_init, create_groups_wasm } from "../pkg/groups_core.js";
init()
    .then(() => {
        tz_groups_init();

        const SPLIT_REGEX = /[\s,"]+/

        // Setup table header.
        let hour_header = document.getElementById("hour-header")
        hour_header.insertCell(0).outerHTML = "<th></th>";
        for (let i = 0; i < 24 * 7; i++) {
            let hour_in_day = (i % 24)
            let twelve_hour_in_day = hour_in_day >= 13 ? hour_in_day - 12 : hour_in_day == 0 ? 12 : hour_in_day
            hour_header.insertCell(i + 1).outerHTML = '<th>' + twelve_hour_in_day + '</th>'
        }

        // Setup table data.
        let schedule_ids = document.getElementById("schedule-ids")
        schedule_ids.oninput = updateTableData
        let timezone = document.getElementById("inputTimezone")
        timezone.onchange = updateTableData

        function updateTableData() {
            let schedules = schedule_ids.value.split(SPLIT_REGEX)
            let table = document.getElementById("schedule-table")
            let table_body = document.getElementById('table-body')

            // Remove any existing data rows.
            var tableHeaderRowCount = 2;
            var rowCount = table.rows.length;
            for (var i = tableHeaderRowCount; i < rowCount; i++) {
                table.deleteRow(tableHeaderRowCount);
            }

            // Add new data rows.
            for (let i = 0; i < schedules.length; i++) {
                let schedule_id = schedules[i]
                if (schedule_id.trim() == "") {
                    continue
                }

                console.log(schedule_id)

                let student = Student.from_encoded(schedule_id)
                if (!student) {
                    continue
                }

                let row = table_body.insertRow()

                let cell = row.insertCell()
                cell.style.textAlign = "left"
                cell.outerHTML = "<th><div style='padding-left: 10px'><div class='text-primary'>" + student.name() + "</div><div class='text-secondary'>" + student.timezone() + "</div></div></th>"

                let timezone = document.getElementById("inputTimezone").value
                let student_avail = student.availability_in_timezone(timezone)
                for (let j = 0; j < 24 * 7; j++) {
                    let cell = row.insertCell()
                    if (student_avail[j] == '1') {
                        cell.classList.add("td_selected")
                    } else {
                        cell.classList.add("td_unselected")
                    }
                }
            }
        }

        // Populate timezone dropdown
        let timezones = timezones_wasm().names
        timezones.forEach(element => {
            $('#inputTimezone').append('<option>' + element + '</option>');
        })
        $('#inputTimezone').selectpicker('refresh')
        $('#inputTimezone').selectpicker('val', 'America/Los_Angeles')

        // Submit button handling
        let submit_button = document.getElementById("submit-button")
        let group_size = document.getElementById("inputGroupSize")
        let spinner = document.getElementById("group-spinner")
        submit_button.onclick = () => {
            spinner.hidden = false

            // Yield so that the button can update before the heavy computation.
            requestAnimationFrame(() =>
                requestAnimationFrame(function () {
                    // Blocks render
                    let schedules = schedule_ids.value.split(SPLIT_REGEX)
                    let groups = create_groups_wasm(schedules, group_size.value)

                    let table_body = document.getElementById('groups-table-body')
                    var i = 1;
                    for (var group of groups) {
                        var row = table_body.insertRow()
                        var cell = row.insertCell()
                        cell.outerHTML = "<td><div>" + i + "</div></td>"

                        cell = row.insertCell()
                        let student_html = "<td><div>"
                        for (var student of group.students) {
                            student_html += Student.from_encoded(student).name() + "<br>"
                        }
                        cell.outerHTML = student_html + "</div></td>"

                        cell = row.insertCell()
                        let suggested_meet_times = ""
                        for (var hour of group.suggested_meet_times) {
                            var day = Math.floor(hour / 24);
                            let hour_in_day = hour % 24;

                            let hour_display = ""
                            if (hour_in_day > 12) {
                                let modded = hour_in_day - 12;
                                hour_display = modded + " PM"
                            } else {
                                if (hour_in_day == 0) {
                                    let modded = 12;
                                    hour_display = modded + " AM"
                                } else {
                                    let modded = hour_in_day
                                    hour_display = modded + " AM"
                                }
                            };

                            let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
                            let day_display = day_names[day]
                            suggested_meet_times += hour_display + " on " + day_display + "<br>"
                        };

                        cell.outerHTML = "<td><div>" + suggested_meet_times + "</div></td>"
                        i ++;
                    }
                    spinner.hidden = true
                }))
        }
    });