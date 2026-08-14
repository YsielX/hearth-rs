return {
    api_version = 1, module_type = "keyword", id = "choose_one", name = "Choose One",
    required_card_hooks = { "on_choose_one" },
    hooks = { on_play = function(ctx, self, target)
        if target == nil then ctx:continue_with("on_choose_one")
        else ctx:continue_with_entity("on_choose_one", target) end
    end },
}
