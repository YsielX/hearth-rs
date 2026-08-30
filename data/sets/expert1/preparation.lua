return {
    api_version = 1,
    id = "EX1_145", rarity = "epic",
    name = "Preparation",
    text = "The next spell you cast this turn costs (2) less.",
    set = "EXPERT1",
    type = "spell",
    class = "rogue",
    cost = 0,
    on_play = function(ctx, self) ctx:set_data(self, "active", 1) end,
    auras = {
        {
            active_zones = { "graveyard" },
            cost = function(ctx, self)
                if ctx:get_data(self, "active") == 1 then return -2 end
                return 0
            end,
            targets = function(ctx, self)
                local result = {}
                if ctx:get_data(self, "active") == 0 then return result end
                for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
                    if ctx:entity(entity).type == "spell" then
                        result[#result + 1] = entity
                    end
                end
                return result
            end,
        },
    },
    triggers = {
        {
            event = "spell_cast",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return ctx:get_data(self, "active") == 1
                    and event.player == ctx:controller(self)
                    and event.player_cast
                    and event.entity ~= self
            end,
            effect = function(ctx, self) ctx:set_data(self, "active", 0) end,
        },
        {
            event = "card_countered",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return ctx:get_data(self, "active") == 1
                    and event.player == ctx:controller(self)
                    and ctx:entity(event.entity).type == "spell"
            end,
            effect = function(ctx, self) ctx:set_data(self, "active", 0) end,
        },
        {
            event = "turn_ended",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return ctx:get_data(self, "active") == 1
                    and event.player == ctx:controller(self)
            end,
            effect = function(ctx, self) ctx:set_data(self, "active", 0) end,
        },
    },
}
