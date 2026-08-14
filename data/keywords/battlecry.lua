return {
    api_version = 1,
    module_type = "keyword",
    id = "battlecry",
    name = "Battlecry",

    required_card_hooks = { "on_battlecry" },
    hooks = {
        on_play = function(ctx, self, target)
            local repetitions = 1
            for _, keyword in ipairs(ctx:entity(self).keywords) do
                if keyword == "battlecry_repeater" then repetitions = 2 break end
            end
            for _ = 1, repetitions do
                if target == nil then
                    ctx:continue_with("on_battlecry")
                else
                    ctx:continue_with_entity("on_battlecry", target)
                end
            end
        end,
    },
}
