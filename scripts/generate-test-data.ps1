# SurrealDB Test Data Generator for Last Mile Shifts
# Generates 3 months of realistic shift data and imports into SurrealDB

# Ensure script is run from project root
if (-not (Test-Path ".\data")) {
    Write-Host "Error: Must be run from project root (data directory not found)" -ForegroundColor Red
    Write-Host "Usage: .\scripts\generate-test-data.ps1" -ForegroundColor Yellow
    exit 1
}

Write-Host "=== SurrealDB Test Data Generator ===" -ForegroundColor Cyan
Write-Host ""

# Configuration
$endpointPath = "file://./data"
$namespace = "lastmile"
$database = "main"
$tempFile = "test-data_temp.surql"

# Generate random shift dates (5 per week, no duplicates)
function Get-ShiftDates {
    param (
        [int]$monthsBack = 3
    )

    $dates = @()
    $today = Get-Date
    $startDate = $today.AddMonths(-$monthsBack)

    # Generate dates by week
    $currentDate = $startDate
    while ($currentDate -le $today) {
        # Add 5 random days from this week
        $weekDays = 0..6 | Get-Random -Count 5
        foreach ($dayOffset in $weekDays) {
            $shiftDate = $currentDate.AddDays($dayOffset)
            if ($shiftDate -le $today) {
                $dates += $shiftDate
            }
        }
        $currentDate = $currentDate.AddDays(7)
    }

    # Sort and return unique dates
    return $dates | Sort-Object | Select-Object -Unique
}

# Generate random ID
function Get-RandomId {
    $chars = 'abcdefghijklmnopqrstuvwxyz0123456789'
    $id = -join ((1..20) | ForEach-Object { $chars[(Get-Random -Maximum $chars.Length)] })
    return "shifts:$id"
}

# Generate shift data
function New-ShiftData {
    param (
        [DateTime]$date,
        [int]$currentOdometer
    )

    # Random shift duration (7-8.5 hours, in increments of 0.25)
    $hoursWorked = (Get-Random -Minimum 28 -Maximum 35) * 0.25

    # Start time (between 7 AM and 2 PM)
    $startHour = Get-Random -Minimum 7 -Maximum 15
    $startMinute = Get-Random -Minimum 0 -Maximum 60
    $startTime = $date.Date.AddHours($startHour).AddMinutes($startMinute)

    # End time
    $endTime = $startTime.AddHours($hoursWorked)

    # Miles driven (80-160 miles per shift)
    $milesDriven = Get-Random -Minimum 80 -Maximum 161
    $odometerStart = $currentOdometer
    $odometerEnd = $odometerStart + $milesDriven

    # Earnings ($30-$60)
    $earnings = [math]::Round((Get-Random -Minimum 3000 -Maximum 6000) / 100, 2)

    # Tips ($35-$85)
    $tips = [math]::Round((Get-Random -Minimum 3500 -Maximum 8500) / 100, 2)

    # Gas cost (roughly $0.08-$0.14 per mile)
    $gasPerMile = (Get-Random -Minimum 8 -Maximum 15) / 100
    $gasCost = [math]::Round($milesDriven * $gasPerMile, 2)

    # Calculate day_total and hourly_pay
    $dayTotal = [math]::Round($earnings + $tips - $gasCost, 2)
    $hourlyPay = [math]::Round($dayTotal / $hoursWorked, 2)

    # Format times for SurrealDB
    $startTimeStr = $startTime.ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
    $endTimeStr = $endTime.ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")

    return @{
        id = Get-RandomId
        day_total = "${dayTotal}dec"
        earnings = "${earnings}dec"
        end_time = "d'$endTimeStr'"
        gas_cost = "${gasCost}dec"
        hourly_pay = "${hourlyPay}dec"
        hours_worked = "${hoursWorked}dec"
        miles_driven = $milesDriven
        notes = "''"
        odometer_end = $odometerEnd
        odometer_start = $odometerStart
        start_time = "d'$startTimeStr'"
        tips = "${tips}dec"
        newOdometer = $odometerEnd
    }
}

# Generate all shifts
Write-Host "Generating shift data..." -ForegroundColor Yellow
$dates = Get-ShiftDates -monthsBack 3
$currentOdometer = 163000  # Starting odometer
$shifts = @()

foreach ($date in $dates) {
    $shift = New-ShiftData -date $date -currentOdometer $currentOdometer
    $shifts += $shift
    $currentOdometer = $shift.newOdometer
}

Write-Host "Generated $($shifts.Count) shifts" -ForegroundColor Green
Write-Host ""

# Create INSERT statement
Write-Host "Creating SurrealDB import file..." -ForegroundColor Yellow
$insertStatement = "INSERT ["

foreach ($shift in $shifts) {
    $shiftObj = @"
    {
        day_total: $($shift.day_total),
        earnings: $($shift.earnings),
        end_time: $($shift.end_time),
        gas_cost: $($shift.gas_cost),
        hourly_pay: $($shift.hourly_pay),
        hours_worked: $($shift.hours_worked),
        id: $($shift.id),
        miles_driven: $($shift.miles_driven),
        notes: $($shift.notes),
        odometer_end: $($shift.odometer_end),
        odometer_start: $($shift.odometer_start),
        start_time: $($shift.start_time),
        tips: $($shift.tips)
    }
"@

    if ($shift -eq $shifts[-1]) {
        $insertStatement += $shiftObj
    } else {
        $insertStatement += $shiftObj + ","
    }
}

$insertStatement += "];"

# Write to file
$insertStatement | Out-File -FilePath $tempFile -Encoding UTF8
Write-Host "Created $tempFile" -ForegroundColor Green
Write-Host ""

# Import into SurrealDB
Write-Host "Importing data into SurrealDB..." -ForegroundColor Yellow
Write-Host "Running: surreal import --endpoint $endpointPath --namespace $namespace --database $database $tempFile" -ForegroundColor Gray
Write-Host ""

try {
    $output = surreal import --endpoint $endpointPath --namespace $namespace --database $database $tempFile 2>&1

    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Import successful!" -ForegroundColor Green
        Write-Host $output
    } else {
        Write-Host "✗ Import failed!" -ForegroundColor Red
        Write-Host $output
        throw "Import command failed with exit code $LASTEXITCODE"
    }
} catch {
    Write-Host "Error during import: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "Keeping $tempFile for debugging" -ForegroundColor Yellow
    exit 1
}

# Clean up temp file
Write-Host ""
Write-Host "Cleaning up temporary files..." -ForegroundColor Yellow
Remove-Item $tempFile -ErrorAction SilentlyContinue
Write-Host "✓ Cleanup complete" -ForegroundColor Green

Write-Host ""
Write-Host "=== Import Complete ===" -ForegroundColor Cyan
Write-Host "Successfully imported $($shifts.Count) shifts into $namespace/$database" -ForegroundColor Green
