return {
    api_version = 1, module_type = "keyword", id = "excavate", name = "Excavate",
    required_card_hooks = { "on_excavate" },
    hooks = { on_play = function(ctx, self)
        local player = ctx:controller(self)
        local total = ctx:get_player_data(player, "excavate_count") + 1
        ctx:set_player_data(player, "excavate_count", total)
        ctx:continue_with_number("on_excavate", ((total - 1) % 4) + 1)
    end },
}
