export type Sample = {
    label: string,
    location_name: string,
    coordinates: string,
    image_path_before: string,
    image_path_after: string,
    measurement: string,
    weight: number | null,

    location_name_check: boolean,
    coordinates_check: boolean,
    image_path_before_check: boolean,
    image_path_after_check: boolean,
    measurement_check: boolean,
    weight_check: boolean,
    all_check: boolean,
}