return {
    api_version = 1, module_type = "keyword", id = "herald", name = "Herald",
    requires_param = true,
    required_card_hooks = { "on_herald" },
    hooks = { on_play = function(ctx, self)
        local player = ctx:controller(self)
        local amount = ctx:keyword_param(self, "herald")
        if amount == nil or amount < 1 then error("herald requires a positive amount") end
        local total = math.min(4, ctx:get_player_data(player, "herald_count") + amount)
        local tier = 0
        if total >= 4 then tier = 2 elseif total >= 2 then tier = 1 end
        ctx:set_player_data(player, "herald_count", total)
        ctx:continue_with_value("on_herald", {
            amount = amount,
            total = total,
            tier = tier,
        })
    end },
}
