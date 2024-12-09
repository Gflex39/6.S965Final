`ifdef SYNTHESIS
`define FPATH(X) `"X`"
`else /* ! SYNTHESIS */
`define FPATH(X) `"../data/X`"
`endif  /* ! SYNTHESIS */

module huffman_ac_lut(
    input wire clk_in,
    input wire rst_in,
    input wire [15:0] code_in,
    input wire [4:0]  code_len_in,
    input wire enable_in,

    output logic valid_out,
    output logic [4:0] codesize_out,
    output logic [4:0] size_out,
    output logic [4:0] run_out
);

    logic [4:0]     stored_code_len;
    logic           stored_valid;

    logic [11:0]    lookup_a;
    logic [3:0]     lookup_run_a;
    logic [3:0]     lookup_size_a;
    logic [3:0]     lookup_codesize_a;

    assign lookup_run_a = lookup_a[11:8];
    assign lookup_size_a = lookup_a[7:4];
    assign lookup_codesize_a = lookup_a[3:0];

    logic [11:0]    lookup_b;
    logic [3:0]     lookup_run_b;
    logic [3:0]     lookup_size_b;
    logic [3:0]     lookup_codesize_b;

    assign lookup_run_b = lookup_b[11:8];
    assign lookup_size_b = lookup_b[7:4];
    assign lookup_codesize_b = lookup_b[3:0];

    logic [11:0]    lookup_c;
    logic [3:0]     lookup_run_c;
    logic [3:0]     lookup_size_c;
    logic [3:0]     lookup_codesize_c;

    assign lookup_run_c = lookup_c[11:8];
    assign lookup_size_c = lookup_c[7:4];
    assign lookup_codesize_c = lookup_c[3:0];

    logic [15:0]    code;
    logic [7:0]     addra;
    logic [6:0]     addrb;
    logic [6:0]     addrc;

    assign code = code_in & ((1 << code_len_in) - 1);

    assign addra = code[7:0];
    assign addrb = code[6:0] & ((1 << (code_len_in - 5)) - 1);
    assign addrc = code[6:0] & ((1 << (code_len_in - 9)) - 1);

    xilinx_single_port_ram_read_first #(
        .RAM_WIDTH(12),
        .RAM_DEPTH(256),
        .RAM_PERFORMANCE("LOW_LATENCY"),
        .INIT_FILE(`FPATH(ac_lut_a.mem))
    ) mba (
        .addra(addra),
        .dina(12'b0),
        .clka(clk_in),
        .wea(1'b0),
        .ena(enable_in),
        .rsta(rst_in),
        .regcea(1'b1),
        .douta(lookup_a)
    );

    xilinx_single_port_ram_read_first #(
        .RAM_WIDTH(12),
        .RAM_DEPTH(128),
        .RAM_PERFORMANCE("LOW_LATENCY"),
        .INIT_FILE(`FPATH(ac_lut_b.mem))
    ) mbb (
        .addra(addrb),
        .dina(12'b0),
        .clka(clk_in),
        .wea(1'b0),
        .ena(enable_in),
        .rsta(rst_in),
        .regcea(1'b1),
        .douta(lookup_b)
    );

    xilinx_single_port_ram_read_first #(
        .RAM_WIDTH(12),
        .RAM_DEPTH(128),
        .RAM_PERFORMANCE("LOW_LATENCY"),
        .INIT_FILE(`FPATH(ac_lut_c.mem))
    ) mbc (
        .addra(addrc),
        .dina(12'b0),
        .clka(clk_in),
        .wea(1'b0),
        .ena(enable_in),
        .rsta(rst_in),
        .regcea(1'b1),
        .douta(lookup_c)
    );

    always_ff @(posedge clk_in) begin
        if (rst_in) begin
            stored_code_len <= 0;
            stored_valid <= 0;
        end else begin
            stored_code_len <= code_len_in;
            stored_valid <= enable_in;
        end
    end

    always_comb begin
        if (stored_code_len < 9) begin
            if (lookup_codesize_a != 0 && stored_code_len == lookup_codesize_a) begin
                valid_out = stored_valid;
                codesize_out = lookup_codesize_a;
                size_out = lookup_size_a;
                run_out = lookup_run_a;
            end else begin
                valid_out = 0;
                codesize_out = 0;
                size_out = 0;
                run_out = 0;
            end
        end else if (stored_code_len < 13) begin
            if (lookup_codesize_b != 0 && stored_code_len == (lookup_codesize_b+9)) begin
                valid_out = stored_valid;
                codesize_out = lookup_codesize_b+9;
                size_out = lookup_size_b;
                run_out = lookup_run_b;
            end else begin
                valid_out = 0;
                codesize_out = 0;
                size_out = 0;
                run_out = 0;
            end
        end else begin
            if (lookup_codesize_c != 0 && stored_code_len == (lookup_codesize_c+13)) begin
                valid_out = stored_valid;
                codesize_out = lookup_codesize_c+13;
                size_out = lookup_size_c;
                run_out = lookup_run_c;
            end else begin
                valid_out = 0;
                codesize_out = 0;
                size_out = 0;
                run_out = 0;
            end
        end
    end

endmodule