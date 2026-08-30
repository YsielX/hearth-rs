local KEY = "frozen_solid"
local EXPIRY = "frozen_solid_expiry"

return {
    api_version = 1,
    module_type = "keyword",
    id = KEY,
    name = "Frozen Solid",
    rules = {
        can_play = function() return false end,
    },
    triggers = {{
        event = "turn_ended",
        timing = "after",
        active_zones = { "hand" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
                and event.turn >= ctx:get_data(self, EXPIRY)
        end,
        effect = function(ctx, self)
            ctx:set_data(self, EXPIRY, 0)
            ctx:disable_keyword(self, KEY)
        end,
    }},
}
