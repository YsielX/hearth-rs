return {
    api_version = 1, module_type = "keyword", id = "manathirst", name = "Manathirst",
    requires_param = true,
    required_card_hooks = { "on_manathirst" },
    hooks = { on_play = function(ctx, self, target)
        if ctx:player(ctx:controller(self)).max_mana < ctx:keyword_param(self, "manathirst") then return end
        if target == nil then ctx:continue_with("on_manathirst")
        else ctx:continue_with_entity("on_manathirst", target) end
    end },
}
