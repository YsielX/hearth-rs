return {
    api_version = 1, module_type = "keyword", id = "invoke", name = "Invoke",
    required_card_hooks = { "on_invoke" },
    hooks = { on_play = function(ctx, self)
        local player = ctx:controller(self)
        local count = ctx:get_player_data(player, "invoke_count") + 1
        ctx:set_player_data(player, "invoke_count", count)
        ctx:continue_with_number("on_invoke", count)
    end },
}
