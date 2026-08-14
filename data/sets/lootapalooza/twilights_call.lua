local function deathrattle_pool(ctx, player)
    local seen, pool = {}, {}
    for _, record in ipairs(ctx:minion_death_records(player)) do
        local deathrattle = false
        for _, keyword in ipairs(record.keywords or {}) do if keyword == "deathrattle" then deathrattle = true; break end end
        if deathrattle and not seen[record.card_id] then seen[record.card_id] = true; pool[#pool + 1] = record.card_id end
    end
    return pool
end
local card = {
    api_version = 1, id = "LOOT_187", name = "Twilight's Call",
    text = "Summon 1/1 copies of 2 friendly <b>Deathrattle</b> minions that died this game.",
    set = "LOOTAPALOOZA", type = "spell", class = "priest", rarity = "rare",
    spell_school = "shadow", cost = 3,
}
function card.on_play(ctx, self)
    ctx:set_data(self, "twilight_left", 2); ctx:continue_with("summon_twilight_copy")
end
function card.summon_twilight_copy(ctx, self)
    if ctx:get_data(self, "twilight_left") <= 0 or #ctx:board(ctx:controller(self)) >= 7 then return end
    local pool = {}
    for _, id in ipairs(deathrattle_pool(ctx, ctx:controller(self))) do
        if ctx:get_data(self, "twilight_used:" .. id) == 0 then pool[#pool + 1] = id end
    end
    if #pool > 0 then ctx:random_value(pool, "receive_twilight_copy") end
end
function card.receive_twilight_copy(ctx, self, id)
    ctx:summon_with_stats(ctx:controller(self), id, 1, 1)
    ctx:set_data(self, "twilight_used:" .. id, 1)
    local left = ctx:get_data(self, "twilight_left") - 1; ctx:set_data(self, "twilight_left", left)
    if left > 0 then ctx:continue_with("summon_twilight_copy") end
end
return card
