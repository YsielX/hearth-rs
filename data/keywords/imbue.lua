return {
    api_version = 1, module_type = "keyword", id = "imbue", name = "Imbue",
    required_card_hooks = { "on_imbue" },
    hooks = { on_play = function(ctx, self)
        local player = ctx:controller(self)
        local count = ctx:get_player_data(player, "imbue_count") + 1
        ctx:set_player_data(player, "imbue_count", count)
        ctx:continue_with_number("on_imbue", count)
    end },
}
