local function cthuns(ctx, player)
    local result = {}
    for _, zone in ipairs({ ctx:hand(player), ctx:deck(player), ctx:board(player), ctx:graveyard(player) }) do
        for _, entity in ipairs(zone) do
            local view = ctx:entity(entity)
            if view.card_id == "OG_280" and not view.silenced then
                result[#result + 1] = entity
            end
        end
    end
    return result
end

return {
    api_version = 1,
    module_type = "keyword",
    id = "cthun_taunt",
    name = "Cthun Taunt",
    auras = {{
        active_zones = { "hero" },
        keywords = { "taunt" },
        targets = function(ctx, self) return cthuns(ctx, ctx:controller(self)) end,
    }},
}
