local card = {
    api_version = 1,
    id = "OG_133",
    name = "N'Zoth, the Corruptor",
    text = "<b>Battlecry:</b> Summon your <b>Deathrattle</b> minions that died this game.",
    set = "OG",
    type = "minion",
    rarity = "legendary",
    cost = 10,
    attack = 5,
    health = 7,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local records = ctx:minion_death_records(ctx:controller(self))
    ctx:set_data(self, "death_record_count", #records)
    ctx:continue_with("summon_next_deathrattle")
end

function card.summon_next_deathrattle(ctx, self)
    local player = ctx:controller(self)
    if #ctx:board(player) >= 7 then return end

    local records = ctx:minion_death_records(player)
    local choices = {}
    local count = ctx:get_data(self, "death_record_count")
    for index = 1, count do
        local record = records[index]
        if record and record.had_deathrattle
            and ctx:get_data(self, "death_record_used_" .. index) == 0 then
            choices[#choices + 1] = index
        end
    end
    if #choices > 0 then
        ctx:random_value(choices, "summon_deathrattle_record")
    end
end

function card.summon_deathrattle_record(ctx, self, index)
    ctx:set_data(self, "death_record_used_" .. index, 1)
    local player = ctx:controller(self)
    local record = ctx:minion_death_records(player)[index]
    if record then ctx:summon(player, record.card_id) end
    ctx:continue_with("summon_next_deathrattle")
end

return card
