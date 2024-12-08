`timescale 1ns / 1ps
`default_nettype none

module huffman_decoder(
    input wire clk_in,
    input wire rst_in,
    input wire serial_in,
    input wire valid_in,

    output logic [10:0] value_out,
    output logic [3:0] run_out,
    output logic dc_out,
    output logic valid_out
);
    parameter S_DC_SIZE = 0;
    parameter S_DC_VALUE = 1;
    parameter S_AC_SIZE = 2;
    parameter S_AC_VALUE = 3;

    logic [2:0]     state;

    logic [26:0]    buffer;
    logic [26:0]    next_buffer;
    logic [4:0]     buffer_len;
    logic [4:0]     next_buffer_len;

    logic           mhdl_valid;
    logic [4:0]     mhdl_codesize;
    logic [4:0]     mhdl_size;

    logic           mhal_valid;
    logic [4:0]     mhal_codesize;
    logic [4:0]     mhal_size;
    logic [4:0]     mhal_run;

    logic [4:0]     dc_value_size;
    logic [4:0]     ac_value_size;
    logic [4:0]     ac_run;

    logic [7:0]     num_decoded;
    logic [7:0]     next_num_decoded;

    logic           next_valid_out;

    huffman_dc_lut mhdl(
        .clk_in(clk_in),
        .rst_in(rst_in),
        .enable_in(1'b1),
        .code_in(buffer[10:0]),
        .code_len_in(buffer_len),
        .valid_out(mhdl_valid),
        .codesize_out(mhdl_codesize),
        .size_out(mhdl_size)
    );

    huffman_ac_lut mhal(
        .clk_in(clk_in),
        .rst_in(rst_in),
        .enable_in(1'b1),
        .code_in(buffer[15:0]),
        .code_len_in(buffer_len),
        .valid_out(mhal_valid),
        .codesize_out(mhal_codesize),
        .size_out(mhal_size),
        .run_out(mhal_run)
    );

    assign dc_out = (state == S_DC_VALUE || state == S_DC_SIZE);

    always_ff @(posedge clk_in) begin
        if (rst_in) begin
            buffer <= 0;
            buffer_len <= 0;
            state <= S_DC_SIZE;
            value_out <= 0;
            valid_out <= 0;
            run_out <= 0;
            dc_value_size <= 0;
            ac_value_size <= 0;
            ac_run <= 0;
            num_decoded <= 0;
        end else begin
            next_buffer_len = buffer_len;
            next_buffer = buffer;
            next_valid_out = 0;
            next_num_decoded = num_decoded;

            case (state)
                S_DC_SIZE: begin
                    if (mhdl_valid) begin
                        if (mhdl_size == 0) begin
                            next_valid_out = 1;
                            run_out <= 0;
                            value_out <= 0;
                        end
                        next_num_decoded = 1;
                        state <= (mhdl_size == 0) ? S_AC_SIZE : S_DC_VALUE;
                        dc_value_size <= mhdl_size;
                        next_buffer_len = next_buffer_len - mhdl_codesize;
                    end
                end

                S_DC_VALUE: begin
                    if (next_buffer_len >= dc_value_size) begin
                        state <= S_AC_SIZE;
                        run_out <= 0;
                        next_valid_out = 1;
                        value_out <= (buffer[10:0] & ((1 << dc_value_size) - 1)) >> (next_buffer_len - dc_value_size);
                        next_buffer_len = next_buffer_len - dc_value_size;
                    end
                end

                S_AC_SIZE: begin
                    if (mhal_valid) begin
                        next_num_decoded = 1 + num_decoded + mhal_run;
                        next_buffer_len = next_buffer_len - mhal_codesize;

                        if (mhal_size == 0 && mhal_run == 0) begin
                            next_valid_out = 1;
                            run_out <= (63-num_decoded);
                            value_out <= 0;
                            state <= S_DC_SIZE;
                        end else begin
                            if (mhal_size == 0) begin
                                next_valid_out = 1;
                                state <= (next_num_decoded >= 64) ? S_DC_SIZE : S_AC_SIZE;
                                run_out <= mhal_run;
                                value_out <= 0;
                            end else begin
                                ac_value_size <= mhal_size;
                                ac_run <= mhal_run;
                                state <= S_AC_VALUE;
                            end
                        end
                    end
                end

                S_AC_VALUE: begin
                    if (next_buffer_len >= ac_value_size) begin
                        state <= (next_num_decoded >= 64) ? S_DC_SIZE : S_AC_SIZE;
                        run_out <= ac_run;
                        next_valid_out = 1;
                        value_out <= (buffer[10:0] & ((1 << ac_value_size) - 1)) >> (next_buffer_len - ac_value_size);
                        next_buffer_len = next_buffer_len - ac_value_size;
                    end
                end
            endcase

            if (valid_in) begin
                next_buffer = {next_buffer[25:0], serial_in};
                next_buffer_len = next_buffer_len + 1;
            end

            buffer_len <= next_buffer_len;
            buffer <= next_buffer;
            valid_out <= next_valid_out;
            num_decoded <= next_num_decoded;
        end
    end

endmodule