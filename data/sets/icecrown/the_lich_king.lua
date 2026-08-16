local death_knight_cards = {
    "ICC_314t1",
    "ICC_314t2",
    "ICC_314t3",
    "ICC_314t4",
    "ICC_314t5",
    "ICC_314t6",
    "ICC_314t7",
    "ICC_314t8",
}

local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords or {}) do
        if keyword == "dormant" then return true end
    end
    return false
end

local frostmourne = {
    id = "ICC_314t1",
    name = "Frostmourne",
    text = "<b>Deathrattle:</b> Summon every minion killed by this weapon.",
    set = "ICECROWN",
    type = "weapon",
    class = "death_knight",
    collectible = false,
    cost = 7,
    attack = 5,
    health = 3,
    keywords = { "deathrattle" },
    triggers = {
        {
            event = "entity_died",
            timing = "after",
            active_zones = { "weapon", "graveyard" },
            condition = function(ctx, self, event)
                local me = ctx:entity(self)
                return event.source == self
                    or event.source == ctx:player(me.controller).hero
            end,
            effect = function(ctx, self, event)
                local count = ctx:get_data(self, "frostmourne_kill_count") + 1
                ctx:set_data(self, "frostmourne_kill_count", count)
                ctx:set_data(self, "frostmourne_kill_" .. count, event.entity)
            end,
        },
    },
}

function frostmourne.on_deathrattle(ctx, self)
    local player = ctx:controller(self)
    for index = 1, ctx:get_data(self, "frostmourne_kill_count") do
        local entity = ctx:get_data(self, "frostmourne_kill_" .. index)
        if entity ~= 0 then ctx:summon(player, ctx:entity(entity).card_id) end
    end
end

local army_of_the_frozen_throne = {
    id = "ICC_314t2",
    name = "Army of the Frozen Throne",
    text = "Remove the top 5 cards of your deck. Summon any minions removed.",
    set = "ICECROWN",
    type = "spell",
    class = "death_knight",
    collectible = false,
    spell_school = "shadow",
    cost = 6,
    -- Requires summoning the original minion entities after moving the top five cards to Removed.
}

function army_of_the_frozen_throne.on_play(ctx, self)
    local player = ctx:controller(self)
    local deck = ctx:deck(player)
    local count = math.min(5, #deck)
    local minions = {}
    for index = 1, count do
        local entity = deck[index]
        if ctx:entity(entity).type == "minion" then
            minions[#minions + 1] = entity
        end
        ctx:move(entity, "removed")
    end
    for _, entity in ipairs(minions) do
        ctx:summon_existing(player, entity)
    end
end

local doom_pact = {
    id = "ICC_314t3",
    name = "Doom Pact",
    text = "[x]Destroy all minions. \nRemove the top card \nfrom your deck for each\nminion destroyed.",
    set = "ICECROWN",
    type = "spell",
    class = "death_knight",
    collectible = false,
    spell_school = "shadow",
    cost = 5,
}

function doom_pact.on_play(ctx, self)
    local minions = {}
    for _, minion in ipairs(ctx:minions()) do
        if not is_dormant(ctx, minion) then minions[#minions + 1] = minion end
    end
    ctx:set_data(self, "doom_pact_destroyed", #minions)
    if #minions > 0 then cardlib.effects.destroy_all(ctx, minions) end
    ctx:continue_with("remove_doom_pact_cards")
end

function doom_pact.remove_doom_pact_cards(ctx, self)
    local deck = ctx:deck(ctx:controller(self))
    local count = math.min(ctx:get_data(self, "doom_pact_destroyed"), #deck)
    for index = 1, count do ctx:move(deck[index], "removed") end
end

local death_grip = {
    id = "ICC_314t4",
    name = "Death Grip",
    text = "Steal a minion from your opponent's deck and add it to your hand.",
    set = "ICECROWN",
    type = "spell",
    class = "death_knight",
    collectible = false,
    spell_school = "shadow",
    cost = 2,
}

function death_grip.on_play(ctx, self)
    local player = ctx:controller(self)
    local candidates = {}
    for _, entity in ipairs(ctx:deck(ctx:opponent(player))) do
        if ctx:entity(entity).type == "minion" then
            candidates[#candidates + 1] = entity
        end
    end
    if #candidates > 0 then
        ctx:random_value(candidates, "steal_death_grip_minion")
    end
end

function death_grip.steal_death_grip_minion(ctx, self, entity)
    ctx:move_to_hand(ctx:controller(self), entity)
end

local death_coil = {
    id = "ICC_314t5",
    name = "Death Coil",
    text = "Deal $5 damage to an enemy, or restore #5 Health to a friendly character.",
    set = "ICECROWN",
    type = "spell",
    class = "death_knight",
    collectible = false,
    spell_school = "shadow",
    cost = 2,
    target_mode = "required",
}

function death_coil.targets(ctx, self)
    local result = {}
    for _, character in ipairs(ctx:enemy_characters(self)) do
        if ctx:entity(character).type == "hero" or not is_dormant(ctx, character) then
            result[#result + 1] = character
        end
    end
    local player = ctx:controller(self)
    result[#result + 1] = ctx:player(player).hero
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if not is_dormant(ctx, minion) then result[#result + 1] = minion end
    end
    return result
end

function death_coil.on_play(ctx, self, target)
    if ctx:entity(target).controller == ctx:controller(self) then
        cardlib.effects.heal(ctx, target, 5)
    else
        cardlib.effects.damage(ctx, target, 5)
    end
end

local obliterate = {
    id = "ICC_314t6",
    name = "Obliterate",
    text = "Destroy a minion. Your hero takes damage equal to its Health.",
    set = "ICECROWN",
    type = "spell",
    class = "death_knight",
    collectible = false,
    cost = 2,
    target_mode = "required",
    targets = function(ctx)
        local result = {}
        for _, minion in ipairs(ctx:minions()) do
            if not is_dormant(ctx, minion) then result[#result + 1] = minion end
        end
        return result
    end,
}

function obliterate.on_play(ctx, self, target)
    local health = math.max(0, ctx:entity(target).health)
    cardlib.effects.destroy(ctx, target)
    cardlib.effects.damage_ignoring_spell_damage(ctx, ctx:player(ctx:controller(self)).hero, health)
end

local anti_magic_shell = {
    id = "ICC_314t7",
    name = "Anti-Magic Shell",
    text = "Give your minions +2/+2 and <b>Elusive</b>.",
    set = "ICECROWN",
    type = "spell",
    class = "death_knight",
    collectible = false,
    spell_school = "shadow",
    cost = 4,
}

function anti_magic_shell.on_play(ctx, self)
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if not is_dormant(ctx, minion) then
            ctx:buff(minion, 2, 2)
            ctx:grant_keyword(minion, "elusive")
        end
    end
end

local death_and_decay = {
    id = "ICC_314t8",
    name = "Death and Decay",
    text = "Deal $3 damage to all enemies.",
    set = "ICECROWN",
    type = "spell",
    class = "death_knight",
    collectible = false,
    spell_school = "shadow",
    cost = 3,
    on_play = function(ctx, self)
        local targets = {}
        for _, character in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(character).type == "hero" or not is_dormant(ctx, character) then
                targets[#targets + 1] = character
            end
        end
        cardlib.effects.damage_all(ctx, targets, 3)
    end,
}

local card = {
    api_version = 1,
    id = "ICC_314",
    name = "The Lich King",
    text = "[x]<b>Taunt</b>\nAt the end of your turn,\nadd a random <b>Lich King</b>\ncard to your hand.",
    set = "ICECROWN",
    type = "minion",
    rarity = "legendary",
    cost = 8,
    attack = 8,
    health = 8,
    tags = { "undead" },
    keywords = { "taunt" },
    triggers = {
        {
            event = "turn_ended",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                ctx:random_value(death_knight_cards, "receive_lich_king_card")
            end,
        },
    },
    tokens = {
        frostmourne,
        army_of_the_frozen_throne,
        doom_pact,
        death_grip,
        death_coil,
        obliterate,
        anti_magic_shell,
        death_and_decay,
    },
}

function card.receive_lich_king_card(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
