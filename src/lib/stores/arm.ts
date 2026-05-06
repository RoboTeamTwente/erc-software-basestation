import { writable } from 'svelte/store';
import type { ArmBoardActualPositions, ArmBoardMovementFeedback } from '$lib/proto/components/arm_board/movement_software_feedback';
import { ArmBoardMovementFeedback_ArmError } from '$lib/proto/components/arm_board/movement_software_feedback';

export const armData = writable<ArmBoardActualPositions>({
  jaw_open: false,
  jaw_actual_position: 0,
  base_actual_position: 0,
  stepper_top_actual_position: 0,
  stepper_bottom_actual_position: 0,
  gripper_rotation_actual_position: 0,
  gripper_pitch_actual_position: 0,
});

export const armFeedback = writable<ArmBoardMovementFeedback>({
  arm_error: ArmBoardMovementFeedback_ArmError.ALL_OK,
});