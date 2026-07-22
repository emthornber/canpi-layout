# canpi-config

Rust library to handle layout definition files for CANPiCap and CANPiZero

This crate provides functionality to read the canpi signalling track diagram configuration file.
This is a JSON file that defines the the various parts of the layout diagram -
CBus linkage, diagram appearance, switch postions, track conformation, and
turnout positions.  The file is validated against a JSON schema generated from
Rust language structures and then loaded internally.

These definitions are then used to create an instance of LayoutDetails.
