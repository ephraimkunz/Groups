import init, { timezones_wasm, Student, groups_core_init_wasm } from "../pkg/groups_core.js"
init()
    .then(() => {
        groups_core_init_wasm()

        // Populate timezone dropdown
        populateTimezoneField()

        // Submit button click handler
        let submit_button = document.getElementById("submit-button")
        submit_button.onclick = generateScheduleCode

        // Make sure double clicking the schedule code selects the whole thing.
        hookUpScheduleCodeDoubleClick()

        // Hook up copy button
        let copy_button = document.getElementById("copy-button")
        copy_button.onclick = copyScheduleCode

        // Handle enabling / disabling the Get Schedule Code button.
        hookUpElementChangeListeners()
    })

function nameField() {
    return document.getElementById("inputName")
}

function timezoneField() {
    return document.getElementById("inputTimezone")
}

function availabilityTable() {
    return document.getElementById("inputAvailability")
}

function scheduleCodeField() {
    return document.getElementById("schedule-code")
}

function populateTimezoneField() {
    let timezones = timezones_wasm()
    timezones.forEach(element => {
        $('#inputTimezone').append('<option>' + element + '</option>')
    })
    $('#inputTimezone').selectpicker('refresh')
}

async function copyScheduleCode(event) {
    const button = event.srcElement
    const pre = button.parentElement
    let code = pre.querySelector("code")
    let text = code.innerText
    await navigator.clipboard.writeText(text)

    button.innerText = "Code Copied"

    setTimeout(() => {
        button.innerText = "Copy"
    }, 1000)
}

function hookUpElementChangeListeners() {
    let element = nameField()
    element.oninput = enableOrDisableGetScheduleCodeButton
    element.onchange = enableOrDisableGetScheduleCodeButton

    element = timezoneField()
    element.onchange = enableOrDisableGetScheduleCodeButton

    element = availabilityTable()
    element.onclick = enableOrDisableGetScheduleCodeButton
}

function enableOrDisableGetScheduleCodeButton() {
    // First, remove any existing schedule code since the inputs changed.
    let schedule_code_outer = document.getElementById("schedule-code").parentElement
    schedule_code_outer.style.visibility = "hidden"


    // Then, check if the button should be enabled.
    let name = nameField().value
    let timezone = timezoneField().value
    let avail = availabilityTable()

    let valid = name.length > 0 && timezone.length > 0

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
        document.getElementById("submit-button").disabled = false
    } else {
        document.getElementById("submit-button").disabled = true
    }
}

function generateScheduleCode() {
    let name = nameField().value
    let timezone = timezoneField().value

    let table = availabilityTable()
    let availability = Array(24 * 7).fill('0')

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
    let schedule_code = scheduleCodeField()
    schedule_code.innerHTML = encoded

    let schedule_code_outer = schedule_code.parentElement
    schedule_code_outer.style.visibility = "visible"
}

function hookUpScheduleCodeDoubleClick() {
    let schedule_code = scheduleCodeField()
    schedule_code.ondblclick = function () {
        event.preventDefault()
        var sel = window.getSelection()
        var range = document.createRange()
        range.selectNodeContents(this)
        sel.removeAllRanges()
        sel.addRange(range)
    }
}