local function dealt_by(ctx, self, event)
    if event.source == self then
        return true
    end
    local me = ctx:entity(self)
    return me.zone == "weapon" and event.source == ctx:player(me.controller).hero
end

local RESOLVING = "lifesteal_resolving"

return {
    api_version = 1,
    module_type = "keyword",
    id = "lifesteal",
    name = "Lifesteal",
    triggers = {
        {
            event = "damaged",
            timing = "after",
            active_zones = { "board", "weapon", "hero_power", "graveyard" },
            condition = function(ctx, self, event)
                return event.amount > 0
                    and ctx:get_data(self, RESOLVING) == 0
                    and dealt_by(ctx, self, event)
            end,
            effect = function(ctx, self, event)
                local me = ctx:entity(self)
                ctx:set_data(self, RESOLVING, 1)
                cardlib.effects.heal(ctx, ctx:player(me.controller).hero, event.amount)
                ctx:set_data(self, RESOLVING, 0)
            end,
        },
    },
}
