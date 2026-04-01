/// ILI9225 instructions.
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    DriverOutputCtrl = 0x01, // Driver Output Control
    LcdAcDrivingCtrl = 0x02, // LCD AC Driving Control
    EntryMode = 0x03,        // Entry Mode
    DispCtrl1 = 0x07,        // Display Control 1
    DispCtrl2 = 0x08,        // Blank Period Control
    FrameCycleCtrl = 0x0B,   // Frame Cycle Control
    InterfaceCtrl = 0x0C,    // Interface Control
    OscCtrl = 0x0F,          // Osc Control
    //*************Power On sequence ****************//
    PowerCtrl1 = 0x10,   // Power Control 1
    PowerCtrl2 = 0x11,   // Power Control 2
    PowerCtrl3 = 0x12,   // Power Control 3
    PowerCtrl4 = 0x13,   // Power Control 4
    PowerCtrl5 = 0x14,   // Power Control 5
    VciRecycling = 0x15, // VCI Recycling
    RamAddrSet1 = 0x20,  // Horizontal GRAM Address Set
    RamAddRSet2 = 0x21,  // Vertical GRAM Address Set
    GramDataReg = 0x22,  // GRAM Data Register
    //-------------- Set GRAM area -----------------//
    GateScanCtrl = 0x30,          // Gate Scan Control Register
    VerticalScrollCtrl1 = 0x31,   // Vertical Scroll Control 1 Register
    VerticalScrollCtrl2 = 0x32,   // Vertical Scroll Control 2 Register
    VerticalScrollCtrl3 = 0x33,   // Vertical Scroll Control 3 Register
    PartialDrivingPos1 = 0x34,    // Partial Driving Position 1 Register
    PartialDrivingPos2 = 0x35,    // Partial Driving Position 2 Register
    HorizontalWindowAddr1 = 0x36, // Horizontal Address Start Position (Window-HStart)
    HorizontalWindowAddr2 = 0x37, // Horizontal Address End Position (Window-HEnd)
    VerticalWindowAddr1 = 0x38,   // Vertical Address Start Position (Window-VStart)
    VerticalWindowAddr2 = 0x39,   // Vertical Address End Position (Window-VEnd)
    // ----------- Adjust the Gamma Curve ----------//
    GammaCtrl1 = 0x50,  // Gamma Control 1
    GammaCtrl2 = 0x51,  // Gamma Control 2
    GammaCtrl3 = 0x52,  // Gamma Control 3
    GammaCtrl4 = 0x53,  // Gamma Control 4
    GammaCtrl5 = 0x54,  // Gamma Control 5
    GammaCtrl6 = 0x55,  // Gamma Control 6
    GammaCtrl7 = 0x56,  // Gamma Control 7
    GammaCtrl8 = 0x57,  // Gamma Control 8
    GammaCtrl9 = 0x58,  // Gamma Control 9
    GammaCtrl10 = 0x59, // Gamma Control 10

    // delay 50ms
    ColorBlack = 0x000000,
    ColorWhite = 0xFFFFFF,
}

