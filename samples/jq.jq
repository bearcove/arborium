# https://github.com/paultag/geoconv-rs/blob/94895406915ec85a2f48eb1692c1b54d2f143a11/src/lib.rs#L702-L789

def pi: 3.14159265358979323846264338327950288;

def earth_radius: 6371008.7714;

def deg2rad:
	if type == "object" then
		map_values(deg2rad)
	elif type == "array" then
		map(deg2rad)
	else
		. * pi / 180
	end
;

def rad2deg:
	if type == "object" then
		map_values(rad2deg)
	elif type == "array" then
		map(rad2deg)
	else
		. * 180 / pi
	end
;

# input: { self: [ lat, lon ], other: [ lat, lon ] }
# output: ^ in radians, plus delta: [ lat, lon ]
def angular_measure_to_radian_delta:
	deg2rad
	| .delta = [
		.other[0] - .self[0],
		.other[1] - .self[1],
		empty
	]
;

# input: { self: [ lat, lon ], other: [ lat, lon ] }
# output: distance in meters
def haversine_distance:
	# https://github.com/paultag/geoconv-rs/blob/94895406915ec85a2f48eb1692c1b54d2f143a11/src/lib.rs#L744-L765
	angular_measure_to_radian_delta as { self: [ $self_lat, $_ ], other: [ $other_lat, $_ ], delta: [ $delta_lat, $delta_lon ] }

	| (
		pow(($delta_lat / 2 | sin); 2)
		+ ($self_lat | cos) * ($other_lat | cos) * pow($delta_lon / 2; 2)
	) as $a

	| (
		2 * atan2(($a | sqrt); (1 - $a | sqrt))
	) as $c

	| earth_radius * $c
;

# input: { self: [ lat, lon ], other: [ lat, lon ] }
# output: bearing in degrees
def bearing:
	# https://github.com/paultag/geoconv-rs/blob/94895406915ec85a2f48eb1692c1b54d2f143a11/src/lib.rs#L770-L789
	angular_measure_to_radian_delta as { self: [ $self_lat, $_ ], other: [ $other_lat, $_ ], delta: [ $_, $delta_lon ] }

	| (
		($self_lat | cos) * ($other_lat | sin)
		- ($self_lat | sin) * ($other_lat | cos) * ($delta_lon | cos)
	) as $x

	| (
		($delta_lon | sin) * ($other_lat | cos)
	) as $y

	| atan2($y; $x)

	# https://github.com/paultag/geoconv-rs/pull/3 was discovered RIGHT HERE 😏
	| rad2deg

	# convert -180 - 180 to 0 - 360
	| if . < 0 then . + 360 else . end
	# (this is ~ "(. + 360) % 360" but preserving floating point decimals)
;

# TODO write some unit tests for this 😅
