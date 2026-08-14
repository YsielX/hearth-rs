return {
    api_version = 1, module_type = "keyword", id = "choose_multiple", name = "Choose Multiple",
    required_card_hooks = { "on_choose_multiple" },
    hooks = { on_play = function(ctx, self, target)
        if target == nil then ctx:continue_with("on_choose_multiple")
        else ctx:continue_with_entity("on_choose_multiple", target) end
    end },
}
