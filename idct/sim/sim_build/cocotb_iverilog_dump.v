module cocotb_iverilog_dump();
initial begin
    $dumpfile("C:/Users/hranw/Documents/GitHub/6.S965Final/idct/sim/sim_build/idct_2d_dma.fst");
    $dumpvars(0, idct_2d_dma);
end
endmodule
