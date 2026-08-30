local function has_keyword(ctx, entity, wanted)
    for _, keyword in ipairs(ctx:entity(entity).keywords or {}) do
        if keyword == wanted then return true end
    end
    return false
end

return {
    api_version = 1,
    module_type = "keyword",
    id = "death_knight_corpses",
    name = "Death Knight Corpses",
    rules = {},
    triggers = {
        {
            event = "game_started",
            timing = "after",
            active_zones = { "hero" },
            effect = function(ctx, self)
                ctx:grant_player_keyword(ctx:controller(self), "death_knight_corpses")
            end,
        },
        {
            event = "entity_died",
            timing = "after",
            active_zones = { "hero" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and not has_keyword(ctx, event.entity, "no_corpse")
            end,
            effect = function(ctx, self, event)
                ctx:gain_resource(event.player, "corpses", 1)
            end,
        },
    },
}
