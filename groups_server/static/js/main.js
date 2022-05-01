(function () {
    'use strict'

    $(function () {
        $('.selectpicker').selectpicker();
    });
})()    

// Handle clicking on time cells.
function select_cell(cell) {
    if (cell.className == "td_unselected")
        cell.className = "td_selected";
    else if (cell.className == "td_selected")
        cell.className = "td_unselected";
}