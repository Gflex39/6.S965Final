// One-dimensional Inverse Discrete Cosine Transform (IDCT) module
// Based on the algorithm outlined in Chen et al "A Fast Computational Algorithm for the Discrete Cosine Transform" :1991
module idct_1d #(
    // Parameterizable bit width for input/output values
    parameter WIDTH = 12
) (
    input logic rst_in,
    input logic clk_in,

    // Input is a packed array of 8 elements, each (WIDTH+4) bits wide
    input logic signed [WIDTH-1:0] idct_in_0,
    input logic signed [WIDTH-1:0] idct_in_1,
    input logic signed [WIDTH-1:0] idct_in_2,
    input logic signed [WIDTH-1:0] idct_in_3,
    input logic signed [WIDTH-1:0] idct_in_4,
    input logic signed [WIDTH-1:0] idct_in_5,
    input logic signed [WIDTH-1:0] idct_in_6,
    input logic signed [WIDTH-1:0] idct_in_7,

    // Output is a packed array of 8 elements, each WIDTH bits wide
    output logic signed [WIDTH-1:0] idct_out_0,
    output logic signed [WIDTH-1:0] idct_out_1,
    output logic signed [WIDTH-1:0] idct_out_2,
    output logic signed [WIDTH-1:0] idct_out_3,
    output logic signed [WIDTH-1:0] idct_out_4,
    output logic signed [WIDTH-1:0] idct_out_5,
    output logic signed [WIDTH-1:0] idct_out_6,
    output logic signed [WIDTH-1:0] idct_out_7
);

  localparam signed [15:0] A = 11585;  // cos(pi / 4) * 2^14;
  localparam signed [15:0] B = 15136;  // cos(pi / 8) * 2^14;
  localparam signed [15:0] C = 6269;  // sin(pi / 8) * 2^14;
  localparam signed [15:0] D = 16069;  // cos(pi / 16) * 2^14;
  localparam signed [15:0] E = 13622;  // cos(3 * pi / 16) * 2^14;
  localparam signed [15:0] F = 9102;  // sin(3 * pi / 16) * 2^14;
  localparam signed [15:0] G = 3196;  // sin(pi / 16) * 2^14;

  // Inputs
  logic signed [WIDTH-1:0] X0, X1, X2, X3, X4, X5, X6, X7;

  logic signed [29:0] im_0, im_2, im_4, im_6;
  logic signed [29:0] im_1, im_3, im_5, im_7;

  always_comb begin : assign_inputs
    X0 = idct_in_0;
    X1 = idct_in_1;
    X2 = idct_in_2;
    X3 = idct_in_3;
    X4 = idct_in_4;
    X5 = idct_in_5;
    X6 = idct_in_6;
    X7 = idct_in_7;
  end

  always_comb begin : intermediate_matrix_operations
    // Stage 1
    // Upper matrix operation
    im_0 = ((A * X0) + (B * X2) + (A * X4) + (C * X6));
    im_2 = ((A * X0) + (C * X2) + (-A * X4) + (-B * X6));
    im_4 = ((A * X0) + (-C * X2) + (-A * X4) + (B * X6));
    im_6 = ((A * X0) + (-B * X2) + (A * X4) + (-C * X6));

    // Lower matrix operation
    im_1 = ((D * X1) + (E * X3) + (F * X5) + (G * X7));
    im_3 = ((E * X1) + (-G * X3) + (-D * X5) + (-F * X7));
    im_5 = ((F * X1) + (-D * X3) + (G * X5) + (E * X7));
    im_7 = ((G * X1) + (-F * X3) + (E * X5) + (-D * X7));
  end

  always_comb begin : assign_outputs
    idct_out_0 = (im_0 + im_1) >>> 15;
    idct_out_1 = (im_2 + im_3) >>> 15;
    idct_out_2 = (im_4 + im_5) >>> 15;
    idct_out_3 = (im_6 + im_7) >>> 15;

    idct_out_7 = (im_0 - im_1) >>> 15;
    idct_out_6 = (im_2 - im_3) >>> 15;
    idct_out_5 = (im_4 - im_5) >>> 15;
    idct_out_4 = (im_6 - im_7) >>> 15;
  end


endmodule
