#!/bin/bash

# SurrealDB Test Data Generator for Last Mile Shifts
# Generates 3 months of realistic shift data and imports into SurrealDB

# Color codes
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
RED='\033[0;31m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

# Ensure script is run from project root
if [ ! -d "./data" ]; then
    echo -e "${RED}Error: Must be run from project root (data directory not found)${NC}"
    echo -e "${YELLOW}Usage: ./scripts/generate-test-data.sh${NC}"
    exit 1
fi

echo -e "${CYAN}=== SurrealDB Test Data Generator ===${NC}"
echo ""

# Configuration
ENDPOINT_PATH="file://./data"
NAMESPACE="lastmile"
DATABASE="main"
TEMP_FILE="test-data_temp.surql"

# Generate random string for ID
generate_random_id() {
    local chars='abcdefghijklmnopqrstuvwxyz0123456789'
    local id=""
    for i in {1..20}; do
        id="${id}${chars:RANDOM%${#chars}:1}"
    done
    echo "shifts:$id"
}

# Generate random number in range
random_range() {
    local min=$1
    local max=$2
    echo $((RANDOM % (max - min) + min))
}

# Generate random float
random_float() {
    local min=$1
    local max=$2
    local value=$((RANDOM % (max - min) + min))
    echo "$value"
}

# Round to 2 decimal places
round_2dp() {
    printf "%.2f" "$1"
}

# Generate shift dates (5 per week, no duplicates)
generate_shift_dates() {
    local months_back=3
    local dates=()

    # Get current date and 3 months ago
    local current_date=$(date +%s)
    local start_date=$(date -d "3 months ago" +%s)

    # Generate dates week by week
    local week_start=$start_date
    while [ $week_start -le $current_date ]; do
        # Generate 5 random days from this week (0-6)
        local days=($(shuf -i 0-6 -n 5))
        for day_offset in "${days[@]}"; do
            local shift_date=$((week_start + day_offset * 86400))
            if [ $shift_date -le $current_date ]; then
                dates+=($shift_date)
            fi
        done
        week_start=$((week_start + 604800)) # Add 7 days
    done

    # Sort and remove duplicates
    printf '%s\n' "${dates[@]}" | sort -n | uniq
}

echo -e "${YELLOW}Generating shift data...${NC}"

# Starting odometer
CURRENT_ODOMETER=163000

# Generate dates
mapfile -t DATES < <(generate_shift_dates)
SHIFT_COUNT=${#DATES[@]}

echo -e "${GREEN}Generated $SHIFT_COUNT shifts${NC}"
echo ""

# Start building INSERT statement
INSERT_STATEMENT="INSERT ["

# Generate shifts
for i in "${!DATES[@]}"; do
    SHIFT_DATE=${DATES[$i]}

    # Random shift duration (7-8.5 hours in 0.25 increments)
    HOURS_QUARTER=$(random_range 28 35)
    HOURS_WORKED=$(echo "scale=2; $HOURS_QUARTER * 0.25" | bc)

    # Start time (7 AM to 2 PM)
    START_HOUR=$(random_range 7 15)
    START_MINUTE=$(random_range 0 60)
    START_TIME=$(date -d "@$SHIFT_DATE" "+%Y-%m-%d")
    START_TIME=$(date -d "$START_TIME $START_HOUR:$START_MINUTE:00" -u "+%Y-%m-%dT%H:%M:%S.000Z")

    # End time (add hours worked)
    HOURS_SECONDS=$(echo "$HOURS_WORKED * 3600" | bc | cut -d. -f1)
    START_EPOCH=$(date -d "$START_TIME" +%s)
    END_EPOCH=$((START_EPOCH + HOURS_SECONDS))
    END_TIME=$(date -d "@$END_EPOCH" -u "+%Y-%m-%dT%H:%M:%S.000Z")

    # Miles driven (80-160)
    MILES_DRIVEN=$(random_range 80 161)
    ODOMETER_START=$CURRENT_ODOMETER
    ODOMETER_END=$((ODOMETER_START + MILES_DRIVEN))

    # Earnings ($30-$60)
    EARNINGS=$(random_range 3000 6000)
    EARNINGS=$(echo "scale=2; $EARNINGS / 100" | bc)

    # Tips ($35-$85)
    TIPS=$(random_range 3500 8500)
    TIPS=$(echo "scale=2; $TIPS / 100" | bc)

    # Gas cost ($0.08-$0.14 per mile)
    GAS_PER_MILE=$(random_range 8 15)
    GAS_COST=$(echo "scale=2; $MILES_DRIVEN * $GAS_PER_MILE / 100" | bc)

    # Calculate day_total and hourly_pay
    DAY_TOTAL=$(echo "scale=2; $EARNINGS + $TIPS - $GAS_COST" | bc)
    HOURLY_PAY=$(echo "scale=2; $DAY_TOTAL / $HOURS_WORKED" | bc)

    # Generate random ID
    SHIFT_ID=$(generate_random_id)

    # Build shift object
    SHIFT_OBJ="    {
        day_total: ${DAY_TOTAL}dec,
        earnings: ${EARNINGS}dec,
        end_time: d'${END_TIME}',
        gas_cost: ${GAS_COST}dec,
        hourly_pay: ${HOURLY_PAY}dec,
        hours_worked: ${HOURS_WORKED}dec,
        id: ${SHIFT_ID},
        miles_driven: ${MILES_DRIVEN},
        notes: '',
        odometer_end: ${ODOMETER_END},
        odometer_start: ${ODOMETER_START},
        start_time: d'${START_TIME}',
        tips: ${TIPS}dec
    }"

    # Add to INSERT statement
    if [ $i -eq $((SHIFT_COUNT - 1)) ]; then
        INSERT_STATEMENT="${INSERT_STATEMENT}${SHIFT_OBJ}"
    else
        INSERT_STATEMENT="${INSERT_STATEMENT}${SHIFT_OBJ},"
    fi

    # Update odometer for next shift
    CURRENT_ODOMETER=$ODOMETER_END
done

INSERT_STATEMENT="${INSERT_STATEMENT}];"

# Write to file
echo -e "${YELLOW}Creating SurrealDB import file...${NC}"
echo "$INSERT_STATEMENT" > "$TEMP_FILE"
echo -e "${GREEN}Created $TEMP_FILE${NC}"
echo ""

# Import into SurrealDB
echo -e "${YELLOW}Importing data into SurrealDB...${NC}"
echo -e "${GRAY}Running: surreal import --endpoint $ENDPOINT_PATH --namespace $NAMESPACE --database $DATABASE $TEMP_FILE${NC}"
echo ""

if OUTPUT=$(surreal import --endpoint "$ENDPOINT_PATH" --namespace "$NAMESPACE" --database "$DATABASE" "$TEMP_FILE" 2>&1); then
    echo -e "${GREEN}✓ Import successful!${NC}"
    echo "$OUTPUT"
else
    echo -e "${RED}✗ Import failed!${NC}"
    echo "$OUTPUT"
    echo ""
    echo -e "${YELLOW}Keeping $TEMP_FILE for debugging${NC}"
    exit 1
fi

# Clean up temp file
echo ""
echo -e "${YELLOW}Cleaning up temporary files...${NC}"
rm -f "$TEMP_FILE"
echo -e "${GREEN}✓ Cleanup complete${NC}"

echo ""
echo -e "${CYAN}=== Import Complete ===${NC}"
echo -e "${GREEN}Successfully imported $SHIFT_COUNT shifts into $NAMESPACE/$DATABASE${NC}"
