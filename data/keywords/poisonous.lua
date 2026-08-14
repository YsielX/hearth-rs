local function dealt_by(ctx, self, event)
    if event.source == self then
        return true
    end
    local me = ctx:entity(self)
    return me.zone == "weapon" and event.source == ctx:player(me.controller).hero
end

return {
    api_version = 1,
    module_type = "keyword",
    id = "poisonous",
    name = "Poisonous",
    triggers = {
        {
            event = "damaged",
            timing = "after",
            active_zones = { "board", "weapon" },
            condition = function(ctx, self, event)
                return event.amount > 0
                    and dealt_by(ctx, self, event)
                    and ctx:entity(event.target).type == "minion"
            end,
            effect = function(ctx, self, event)
                ctx:destroy(event.target)
            end,
        },
    },
}
