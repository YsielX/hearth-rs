return {
    api_version = 1, module_type = "keyword", id = "outcast", name = "Outcast",
    required_card_hooks = { "on_outcast" },
    hooks = { on_play = function(ctx, self, target)
        if not ctx:outcast_active(self) then return end
        if target == nil then ctx:continue_with("on_outcast")
        else ctx:continue_with_entity("on_outcast", target) end
    end },
}
