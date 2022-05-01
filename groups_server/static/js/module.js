import init, { timezones_wasm, Student, tz_groups_init } from "../pkg/groups_core.js";
init()
    .then(() => {
        tz_groups_init();

        // Populate timezone dropdown
        let timezones = timezones_wasm().names
        timezones.forEach(element => {
            $('#inputTimezone').append('<option>' + element + '</option>');
        })
        $('#inputTimezone').selectpicker('refresh')

        // Submit button handling
        function get_schedule_code() {
            let name = document.getElementById("inputName").value;
            let timezone = document.getElementById("inputTimezone").value;

            let table = document.getElementById("inputAvailability")
            let availability = Array(24 * 7).fill('0');

            // Skip the header row.
            for (let i = 1; i < table.rows.length; i++) {
                let row = table.rows[i]
                for (let j = 0; j < row.cells.length; j++) {
                    let cell = row.cells[j]

                    let classname = cell.className
                    if (classname == "td_selected") {
                        for (let k = 0; k < 4; k++) {
                            availability[7 + (24 * j) + (4 * (i - 1)) + k] = '1'
                        }
                    } else if (classname == "td_unselected") {
                        for (let k = 0; k < 4; k++) {
                            availability[7 + (24 * j) + (4 * (i - 1)) + k] = '0'
                        }
                    }
                }
            }

            let avail = availability.join('')
            let student = new Student(name, timezone, avail)
            let encoded = student.encode()

            console.log(encoded)
            let schedule_code = document.getElementById("schedule-code")
            schedule_code.innerHTML = encoded

            let schedule_code_outer = schedule_code.parentElement
            schedule_code_outer.style.visibility = "visible"
        }

        let submit_button = document.getElementById("submit-button")
        submit_button.onclick = get_schedule_code

        // Make sure double clicking the schedule code selects the whole thing.
        let schedule_code = document.getElementById("schedule-code")
        schedule_code.ondblclick = function () {
            event.preventDefault();
            var sel = window.getSelection();
            var range = document.createRange();
            range.selectNodeContents(this);
            sel.removeAllRanges();
            sel.addRange(range);
        };

        // Hook up copy button
        let copy_button = document.getElementById("copy-button")
        copy_button.addEventListener("click", copyCode);

        async function copyCode(event) {
            const button = event.srcElement;
            const pre = button.parentElement;
            let code = pre.querySelector("code");
            let text = code.innerText;
            await navigator.clipboard.writeText(text);

            button.innerText = "Code Copied";

            setTimeout(() => {
                button.innerText = "Copy";
            }, 1000)
        }

        // Handle enabling / disabling the Get Schedule Code button.
        let element = document.getElementById("inputName")
        element.oninput = validateGetSchedulerCodeButton;
        element.onchange = validateGetSchedulerCodeButton;

        element = document.getElementById("inputTimezone")
        element.onchange = validateGetSchedulerCodeButton;

        element = document.getElementById("inputAvailability")
        element.onclick = validateGetSchedulerCodeButton;

        function validateGetSchedulerCodeButton() {
            // First, remove any existing schedule code since the inputs changed.
            let schedule_code_outer = document.getElementById("schedule-code").parentElement
            schedule_code_outer.style.visibility = "hidden"


            // Then, check if the button should be enabled.
            console.log("validateGetSchedulerCodeButton")
            let name = document.getElementById("inputName").value;
            let timezone = document.getElementById("inputTimezone").value;
            let avail = document.getElementById("inputAvailability")

            let valid = name.length > 0 && timezone.length > 0;

            let table_valid = false
            for (let i = 1; i < avail.rows.length; i++) {
                let row = avail.rows[i]
                for (let j = 0; j < row.cells.length; j++) {
                    let cell = row.cells[j]

                    let classname = cell.className
                    if (classname == "td_selected") {
                        table_valid = true
                        break
                    }
                }
            }

            valid &&= table_valid

            if (valid) {
                document.getElementById("submit-button").disabled = false;
            } else {
                document.getElementById("submit-button").disabled = true;
            }
        }
    });