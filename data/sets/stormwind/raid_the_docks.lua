local function is_pirate(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "pirate" then return true end
    end
    return false
end

local function draw_weapon(ctx, player)
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).type == "weapon" then
            ctx:move(entity, "hand")
            return
        end
    end
end

local function fire_cannon(ctx, self, resume_hook)
    local targets = ctx:enemy_characters(self)
    if #targets > 0 then ctx:random_entity(targets, resume_hook) end
end

local card = {
    api_version = 1,
    id = "SW_028",
    name = "Raid the Docks",
    text = "[x]<b>Questline:</b> Play 3 Pirates.\n<b>Reward:</b> Draw a weapon.",
    set = "STORMWIND",
    type = "spell",
    class = "warrior",
    cost = 1,
    keywords = { "questline" },
    triggers = {
        {
            event = "minion_played",
            timing = "after",
            active_zones = { "secret" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self) and is_pirate(ctx, event.entity)
            end,
            effect = function(ctx, self)
                local progress = ctx:get_data(self, "progress") + 1
                ctx:set_data(self, "progress", progress)
                if progress == 3 then
                    draw_weapon(ctx, ctx:controller(self))
                elseif progress == 6 then
                    ctx:set_data(self, "cannons", 0)
                    fire_cannon(ctx, self, "on_quest_cannon")
                elseif progress == 9 then
                    ctx:reveal_secret(self)
                    ctx:give_card(ctx:controller(self), "SW_028t5")
                end
            end,
        },
    },
}

function card.on_quest_cannon(ctx, self, target)
    cardlib.effects.damage(ctx, target, 2)
    local cannons = ctx:get_data(self, "cannons") + 1
    ctx:set_data(self, "cannons", cannons)
    if cannons < 2 then fire_cannon(ctx, self, "on_quest_cannon") end
end

card.tokens = {
    {
        id = "SW_028t",
        name = "Create a Distraction",
        text = "[x]<b>Questline:</b> Play 3 Pirates.\n<b>Reward:</b> Deal $2 damage\nto a random enemy twice.",
        set = "STORMWIND", type = "spell", class = "warrior", cost = 1,
    },
    {
        id = "SW_028t2",
        name = "Secure the Supplies",
        text = "[x]<b>Questline:</b> Play 3 Pirates.\n<b>Reward:</b> Cap'n Rokara.",
        set = "STORMWIND", type = "spell", class = "warrior", cost = 1,
    },
    {
        id = "SW_028t5",
        name = "Cap'n Rokara",
        text = "<b>Battlecry:</b> Summon The Juggernaut!",
        set = "STORMWIND", type = "minion", class = "warrior",
        cost = 5, attack = 7, health = 7, tags = { "pirate" },
        keywords = { "battlecry" },
        on_battlecry = function(ctx, self)
            ctx:summon(ctx:controller(self), "SW_028t6")
        end,
    },
    {
        id = "SW_028t6",
        name = "The Juggernaut",
        text = "[x]<b>Start of Your Turn:</b>\nSummon a Pirate, equip a\nWarrior weapon, and fire two\n cannons that deal 2 damage!",
        set = "STORMWIND", type = "minion", class = "warrior",
        cost = 5, attack = 0, health = 1,
        triggers = {
            {
                event = "turn_started", timing = "after", active_zones = { "board" },
                condition = function(ctx, self, event)
                    return event.player == ctx:controller(self)
                end,
                effect = function(ctx, self)
                    local pirates = {}
                    for _, card_id in ipairs(ctx:collectible_cards()) do
                        local definition = ctx:card_definition(card_id)
                        for _, tag in ipairs(definition.tags) do
                            if tag == "pirate" then
                                pirates[#pirates + 1] = card_id
                                break
                            end
                        end
                    end
                    if #pirates > 0 then ctx:random_value(pirates, "on_juggernaut_pirate") end
                end,
            },
        },
        on_juggernaut_pirate = function(ctx, self, card_id)
            ctx:summon(ctx:controller(self), card_id)
            local weapons = {}
            for _, candidate in ipairs(ctx:collectible_cards()) do
                local definition = ctx:card_definition(candidate)
                if definition.type == "weapon" and definition.class == "warrior" then
                    weapons[#weapons + 1] = candidate
                end
            end
            if #weapons > 0 then
                ctx:random_value(weapons, "on_juggernaut_weapon")
            else
                ctx:set_data(self, "cannons", 0)
                fire_cannon(ctx, self, "on_juggernaut_cannon")
            end
        end,
        on_juggernaut_weapon = function(ctx, self, card_id)
            ctx:equip_weapon(ctx:controller(self), card_id)
            ctx:set_data(self, "cannons", 0)
            fire_cannon(ctx, self, "on_juggernaut_cannon")
        end,
        on_juggernaut_cannon = function(ctx, self, target)
            cardlib.effects.damage(ctx, target, 2)
            local cannons = ctx:get_data(self, "cannons") + 1
            ctx:set_data(self, "cannons", cannons)
            if cannons < 2 then fire_cannon(ctx, self, "on_juggernaut_cannon") end
        end,
    },
}

return card
