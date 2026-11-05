# infinitier_plt_resource

Reader for PLT, the paperdoll image the Infinity Engine draws on its inventory and
character-generation screens.

A PLT holds no colours. Each pixel names one of a creature's colours and a shade of it, so the
same file paints differently for every creature: the renderer looks each pixel up in the gradient
the creature's colour selects (`MPAL256.BMP` in the games that ship PLTs).
