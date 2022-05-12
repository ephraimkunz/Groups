import init, { timezones_wasm, Student, tz_groups_init, create_groups_wasm } from "../pkg/groups_core.js";
init()
    .then(() => {
        tz_groups_init();

        const SPLIT_REGEX = /[\s,"]+/

        const DEFAULT_TIMEZONE = 'America/Los_Angeles';

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
        $('#inputTimezone').selectpicker('val', DEFAULT_TIMEZONE)

        // Submit button handling
        let submit_button = document.getElementById("submit-button")
        let group_size = document.getElementById("inputGroupSize")
        let spinner = document.getElementById("group-spinner")
        submit_button.onclick = () => {
            let output_timezone = document.getElementById("inputTimezone").value;

            // Remove any existing data rows.
            var tableHeaderRowCount = 1;
            var table = document.getElementById("groups-table");
            var rowCount = table.rows.length;
            for (var i = tableHeaderRowCount; i < rowCount; i++) {
                table.deleteRow(tableHeaderRowCount);
            }

            spinner.hidden = false

            // Yield so that the button can update before the heavy computation.
            requestAnimationFrame(() =>
                requestAnimationFrame(function () {
                    // Blocks render
                    let schedules = schedule_ids.value.split(SPLIT_REGEX)
                    let groups = create_groups_wasm(schedules, group_size.value, output_timezone)

                    // Update schedule ids (and by extension the table)
                    let new_schedule_ids = []
                    for (var group of groups) {
                        for (var student of group.students) {
                            new_schedule_ids.push(student)
                        }
                    }
                    schedule_ids.value = new_schedule_ids.join("\n")
                    updateTableData()

                    // Add new data rows.
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
                        for (var string of group.suggested_meet_times) {
                            suggested_meet_times += string + " (" + output_timezone + ")<br>"
                        };

                        cell.outerHTML = "<td><div>" + suggested_meet_times + "</div></td>"
                        i++;
                    }
                    spinner.hidden = true
                }))
        }
    });